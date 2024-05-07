- chat operate on a model of chatrooms
- chatrooms are hosted by a host
- each user has a id and a name bound to it
- chat is a separate entity
- chat updates itself and exposes the contents
- the user can tell the chat to send a message
*server*
- Spawn a new thread for each incoming connection.
- Each thread handles reading messages from the client and broadcasting them to all other clients.
- Use a channel to communicate between threads to avoid data races.

*client*
- Use asynchronous I/O to handle sending and receiving messages concurrently.
- Implement a simple protocol for communication with the server.
