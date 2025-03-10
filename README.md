# gRPC Registry RUST Micro-service


This is a gRPC registry designed to keep track of multiple gRPC micro services. 
Each participating service registers by protobuf name. It is used by a gRPC client
to find a matching gRPC service by protobuf name. 

It's primary purpose is to support gRPC services in a container management system such
as Kubernetes. It can also function as a simple load balancer across
multiple services offering the same protobuf configurations.

### Functions
```
AUTHORIZE  - obtains a JWT token for use by clients to perform FIND

REGISTER   - allows a service to register availability by protobuf

DEREGISTER - allows a service to remove it's availability

KEEPALIVE  - a service periodically reports it's availability 

FIND       - find a service with a matching protobuf configuration
```
#### Optional 
The FIND function has the added ability to serve as a load balancer, either
round-robin or by minimum load as reported by KEEPALIVE

### Security
This service uses JWT authentication tokens for clients to FIND services
or for services to perform DEREGISTER or KEEPALIVE requests.

## License
Apache