import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function middleware(request: NextRequest) {
  const response = NextResponse.next({
    request: {
      headers: new Headers(request.headers),
    },
  });
  response.headers.set("cache-control", "public, max-age=31536000");
  response.headers.set("cdn-cache-control", "public, max-age=31536000");

  return response;
}

export const config = {
  matcher: ["/:path*"],
};
