# Network

# TODO

- Client Struct on server side that hold the seq number/IP address/port and the state we supposed the client knows
- Create Delta struct that hold the difference between two ECS
- Finish compute and apply delta functions
- Create message to hold the delta and previous state ID

## first version
- Simple protocol
- Connection client/server (naive? Ip/port only is ok)
- Send all network state at every loop
- Out of order packet are discarded

## Later
Do we need to send delta instead?
What is the packet size?
Try to answer those questions.
- Rate limiter on server side?

# Design?

follow common Client-Server design.
The client is a dumb software that just renders and send commands to the server.
For example, "move forward", "shoot" and so on.

The engine does not render anything but it will be holding the state of the system.
The true state, so all updates from engine to clients are to be applied. In the engine,
physics system and so on are running.

## How many clients per server?
8 to begin with? The engine will accept connections as long as it has free slots.
Clients have to be identified by ip/port as multiple clients from the same house
can connect. See Carmack stuff for NAT bugs...


## What is a server?
It represents a current game. Could be called a room in some FPS. The
server will have some game information (what level, what mode and so on).

## Client loop

Does the order matter here? Sends command as fast as possible (30/60 times per sec)
loop {
        get server events
        render
        send commands to server
}

## Server loop

Same question as above.
loop {
        fetch updates
        run systems (no rendering)
        send state
}

How often do we send the state?

## What to send?
Try the quake way. The server is sending snapshots of state. To avoid sending
too much, it takes a snapshot each frame, then send the difference (delta)
between that state and the last known state of the client. The client needs to
include its current state when sending command so that the server can know it.

## How client connect to server

Client send a connection request until it receives a response from the server.
It will timeout if not response.

When server receives, it will take a look at its slots:
- If client is already connected, send connection accepted
- If client is not connected:
        - If a slot is available, send connection accepted
        - If no slot is available, send connection refused
The server does not need to send multiple messages. Only the client does.
If the server does not hear from the client in X second (5?), it will consider
it disconnected and kick it out of the server :)
