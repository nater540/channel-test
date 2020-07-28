# Channel Test

Extremely basic example that uses a [Tokio](https://tokio.rs/) MPSC (*M*ulti*P*roducer*S*ingle*C*onsumor) channel to send events between threads and [Lapin](https://github.com/CleverCloud/lapin) for publishing & consuming AMQP messages.

## Development

1) (Optional) Start the RabbitMQ container ->
  ```shell
  docker-compose up
  ```

2) Compile the rust project ->
  ```shell
  cargo run
  ```

3) Watch with complete and utter awe as messages are sent & received!
