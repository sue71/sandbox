import react from "@vitejs/plugin-react";
import { visualizer } from "rollup-plugin-visualizer";
import vike from "vike/plugin";
import { UserConfig } from "vite";

const config: UserConfig = {
  plugins: [react(), vike(), visualizer()],
};

export default config;
