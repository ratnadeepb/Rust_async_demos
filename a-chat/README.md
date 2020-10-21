# A chat server demonstratong async-std

## Specification

This is a simple text protocol over TCP.The protocol consists of utf-8 messages, separated by `\n`.<br>
The client connects to the server and sends login as a first line. After that, the client can send messages to other clients using the following syntax:
```none
login1, login2, ... loginN: message
```
Each of the specified clients then receives a from login: message message.<br>
A possible session:
```none
On Alice's computer:   |   On Bob's computer:

> alice                |   > bob
> bob: hello               < from alice: hello
                       |   > alice, bob: hi!
                           < from bob: hi!
< from bob: hi!        |
```
The main challenge for the chat server is keeping track of many concurrent connections. The main challenge for the chat client is managing concurrent outgoing messages, incoming messages and user's typing.