package main

import (
	"context"
	"log"
	"net"
	"os"
	"os/signal"

	helloworldpb "github.com/sue71/sandbox/typescript/envoy-grpc-web/backend/internal/proto/helloworld"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

type server struct {
	helloworldpb.UnimplementedGreeterServer
}

func (s *server) SayHello(ctx context.Context, req *helloworldpb.HelloRequest) (*helloworldpb.HelloResponse, error) {
	name := req.Name
	response := &helloworldpb.HelloResponse{
		Message: "Hello " + name,
	}
	return response, nil
}

func main() {
	// Create a listener on TCP port
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalln("Failed to listen:", err)
	}

	// Create a gRPC server object
	s := grpc.NewServer()
	// Attach the Greeter service to the server
	helloworldpb.RegisterGreeterServer(s, &server{})
	// Server reflection
	reflection.Register(s)
	// Serve gRPC server
	log.Println("Serving gRPC on connection ")
	go func() {
		log.Println("Start gRPC server")
		log.Fatalln(s.Serve(lis))
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit
	log.Println("stopping gRPC server...")
	s.GracefulStop()
}