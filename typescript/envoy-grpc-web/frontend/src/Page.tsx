import { FC } from "react";
import { useQuery } from "@connectrpc/connect-query";
import { sayHello } from "./gen/proto/helloworld/service-Greeter_connectquery";
import { HelloRequest } from "./gen/proto/helloworld/service_pb";

export const Page: FC = () => {
  const { data } = useQuery(sayHello, new HelloRequest({ name: "world" }));
  return <div>{data?.message}</div>;
};
