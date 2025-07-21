# Networking support

**Async runtime** aims to provide easy way to use it applications that were previously using `OSS` crates for runtimes like `tokio` or `async_std` still targeting automotive software grade.

## Layered view

![layers](assets/io_layers.drawio.svg)

## WoW explainer

```plantuml


actor User

participant UdpSocket
participant MIO
participant Worker
participant OS
participant Network

note left of User
User calls network API
in some spawned Task
end note

User -> UdpSocket: connect()
UdpSocket -> OS: connect()

User -> UdpSocket: recv().await
UdpSocket -> MIO: recv_from_fd()
UdpSocket <-- MIO: non blocking return

...
note left
Until there is nothing to **recv** worker will process other
**tasks**, ie. other **recv** on different sockets. So the **SAME**
thread can meanwhile receive from other sources, or do anything else.
end note


loop
Worker -> Worker: process_tasks
Worker -> MIO: process_io

loop for each ready FD
MIO -> MIO: waiting_task.wake()
end

alt UdpSocket task was waked
Worker -> Worker: execute_task()
Worker --> UdpSocket: recv()

UdpSocket --> User: received_data

end


```
