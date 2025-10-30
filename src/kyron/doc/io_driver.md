# Networking support

**Kyron** aims to provide easy way to use it applications that were previously using `OSS` crates for runtimes like `tokio` or `async_std` still targeting automotive software grade.

## Layered view

![layers](assets/io_layers.drawio.svg)

## IoDriver integration

### Class diagram for interaction between objects

```plantuml

class IoDriver
class IoDriverHandle

class RegistrationInfo
class AsyncRegistration
class BridgedFd


BridgedFd *-- AsyncRegistration: 1
AsyncRegistration *-- RegistrationInfo: 1

AsyncRegistration o-- IoDriverHandle
note right of AsyncRegistration
Connect MIO object into IoDriver and RegistrationInfo
and delivers API that can let upper code know when
some events are ready.
end note

IoDriver o-- RegistrationInfo: 0..N
note left of IoDriver
- Keeps lifetime of RegistrationInfo
- Pools MIO selector to find out ready IO events
- Wakes tasks that requested event interest via AsyncRegistration
end note


note bottom of RegistrationInfo
Information exchange between IoDriver and AsyncRegistration
to be able to provide user level API for async IO
end note



note top of BridgedFd
Provides async/poll API for IO sources like
TcpListener, UdpSocket etc so they can create
async API without knowing any registration details
end note
```

### Sequence diagrams

Keep in mind that this is simplified logic show here for now.

#### Registering IO source into driver

```plantuml
participant TcpListenerExample
participant BridgedFd


TcpListenerExample -> BridgedFd **: new(mio_source)
BridgedFd -> AsyncRegistration **: new(&mut mio_source)
AsyncRegistration -> IoDriverHandle: add_io_source(&mut mio_source, IoEventInterest)
IoDriverHandle -> RegistrationInfo **
IoDriverHandle -> IoDriverHandle: internal_logic
IoDriverHandle -> MIORegistry: register(&mut mio_source, IoEventInterest, addr_of(RegistrationInfo) aka REGISTRATION_INFO_ADDR)

IoDriverHandle --> AsyncRegistration: Arc<RegistrationInfo>
AsyncRegistration --> BridgedFd
BridgedFd --> TcpListenerExample

```

#### Processing IO events

```plantuml

participant Worker

Worker -> IoDriver: process_io()

IoDriver -> IoDriver: cleanup_registration_infos()

IoDriver -> MIO: poll(&mut events)

loop event in events

alt event.id != UNPARKER_ID

    IoDriver -> IoDriver: recreate_registration_info(event.id)
    note right of IoDriver
        Here event.id is REGISTRATION_INFO_ADDR and we do
        integer to ptr conversion. The lifetime specifics are
        described in code
    end note
    IoDriver -> RegistrationInfo: wake(event)
    RegistrationInfo -> RegistrationInfo: mark_readiness_state(event)

    loop FOR_ALL_REGISTERED_WAKERS
        RegistrationInfo -> Waker: wake_by_ref()
        note right of Waker
            Wakeup over rust Waker
    end note
    end

    RegistrationInfo --> IoDriver
end

end

```

### IO call from upper layer

```plantuml

participant TcpListener
-> TcpListener: async fn accept()
TcpListener -> BridgedFd ++: async_call(IoEventInterest, f) where f is clouser

loop
    BridgedFd -> AsyncRegistration: request_readiness(IoEventInterest).await
    ...
    note right of AsyncRegistration
    Async wait until IoDriver does not
    notify via waker that interest for this IO
    is ready
    end note
    BridgedFd -> BridgedFd ++: f()
    BridgedFd -> OS: accept(O_NONBLOCK)
    BridgedFd --> BridgedFd --

    alt OS call returned WouldBlock
        BridgedFd -> AsyncRegistration: clear_readiness() && continue
        note right of BridgedFd
        We need to clear consumed readiness so we can
        be notified next time
        end note
    else OS call returned data || OS call returned other error
        BridgedFd --> TcpListener: Ok() or Err() && break
    end
end

 TcpListener -> TcpListener



```
