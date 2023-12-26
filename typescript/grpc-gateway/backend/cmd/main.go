package main

import (
	"context"
	"log"
	"net"
	"net/http"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/rs/cors"
	helloworldpb "github.com/sue71/sandbox/typescript/grpc-gateway/backend/internal/proto/helloworld"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
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
	lis, err := net.Listen("tcp", ":8081")
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

	// Create a client connection to the gRPC server we just started
	conn, err := grpc.DialContext(
        context.Background(),
        "0.0.0.0:8081",
        grpc.WithBlock(),
        grpc.WithTransportCredentials(insecure.NewCredentials()),
    )
    if err != nil {
        log.Fatalln("Failed to dial server:", err)
    }
	defer conn.Close()

	mux := runtime.NewServeMux()
	// Register Greeter
	err = helloworldpb.RegisterGreeterHandler(context.Background(), mux, conn)
	if err != nil {
		log.Fatalln("Failed to register gateway:", err)
	}
	withCors := cors.New(cors.Options{
		AllowOriginFunc:  func(origin string) bool { return true },
		AllowedMethods:   []string{"GET", "POST", "PATCH", "PUT", "DELETE", "OPTIONS"},
		AllowedHeaders:   []string{"ACCEPT", "Authorization", "Content-Type", "X-CSRF-Token"},
		ExposedHeaders:   []string{"Link"},
		AllowCredentials: true,
		MaxAge:           300,
	  }).Handler(mux)
	  
	gwServer := &http.Server{
		Addr:    ":8090",
		Handler: withCors,
	}

	log.Println("Serving gRPC-Gateway on connection on http://0.0.0.0:8090")
	log.Fatalln(gwServer.ListenAndServe())
}