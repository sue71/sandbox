import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  experimental: {
    swcPlugins: [
      ["./replace_t.wasm", { "base_dir": "public/locales" }]
    ]
  }
};

export default nextConfig;
