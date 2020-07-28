use tokio_amqp::*;
use futures_util::stream::StreamExt;
use tokio::time::{delay_for, Duration};
use tokio::sync::mpsc::channel as mpsc_channel;
use lapin::{options::*, Connection, ConnectionProperties, types::FieldTable, BasicProperties};

mod error;
pub use error::*;

#[derive(Debug)]
struct Event {
  pub message: String
}

impl Event {
  pub fn new(message: String) -> Self {
    Event { message }
  }
}

#[tokio::main(core_threads = 6)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let address    = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://fluffy:bunny@127.0.0.1:5672/%2f".into());
  let connection = Connection::connect(&address, ConnectionProperties::default().with_tokio()).await?;

  // Create two channels - one for sending and another for receiving
  let channel_a = connection.create_channel().await?;
  let channel_b = connection.create_channel().await?;

  // Create the simple queue
  channel_a.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default()).await?;

  // Create a mpsc channel to send data between threads
  let (mut tx, mut rx) = mpsc_channel::<Event>(100);

  // Create the message send task
  let bg_send_task = Box::pin(async move {
    loop {
      while let Some(event) = rx.recv().await {
        println!("Sending: {:?}", event);

        channel_a.basic_publish(
          "",                             // Exchange
          "hello",                        // Routing Key
          BasicPublishOptions::default(), // Publish options
          event.message.into_bytes(),     // Payload
          BasicProperties::default()      // Properties
        )
        .await
        .expect("Womp Womp");
      }
    }
  });

  // Spawn the message send task
  tokio::spawn(bg_send_task);

  // Create the AMQP consumer
  let mut consumer = channel_b.basic_consume(
    "hello",                          // Exchange
    "my_consumer",                    // Consumer tag
    BasicConsumeOptions::default(),   // Consume options
    FieldTable::default()             // Arguments
  )
  .await?;

  // Create the message receive task
  let bg_recv_task = Box::pin(async move {
    while let Some(delivery) = consumer.next().await {
      let (channel, delivery) = delivery.expect("error in consumer");

      // Deserialize the payload as a string
      let data = String::from_utf8(delivery.data).expect("Insert overly dramatic explosion sounds here...");
      println!("Received: {:?}", data);

      // Send the delivery acknowledgment
      channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await.expect("ack");
    }
  });

  // Spawn the message receive task
  tokio::spawn(bg_recv_task);

  for i in 0..10 {
    if let Err(_) = tx.send(Event::new(format!("Super Kawaii #{}", i))).await {
      println!("receiver dropped");
    }
  }

  // Sleep - for great justice!
  // Yeah this is a ...less than ideal... solution, but it works for testing purposes!
  delay_for(Duration::from_millis(2000)).await;
  println!("Work Complete!");

  Ok(())
}
