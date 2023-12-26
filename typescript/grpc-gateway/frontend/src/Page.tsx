import { FC } from "react";
import { useQuery } from "@tanstack/react-query";

export const Page: FC = () => {
  const { data } = useQuery({
    queryKey: ["hello"],
    queryFn: async () => {
      const response = await fetch("http://localhost:8090/v1/message", {
        method: "POST",
        body: JSON.stringify({ name: "World" }),
      });
      return response.json();
    },
  });
  return <div>{data?.message}</div>;
};
