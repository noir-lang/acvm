import { esbuildPlugin } from "@web/dev-server-esbuild";
import { playwrightLauncher } from "@web/test-runner-playwright";

export default {
  browsers: [playwrightLauncher({ product: "chromium" })],
  plugins: [
    esbuildPlugin({
      ts: true,
    }),
  ],
  files: ["test/browser/**/*.test.ts"],
  nodeResolve: true,
  testRunnerHtml: (testFramework) => `
  <html>
    <head>
      <script type="module" src="${testFramework}"></script>
      <script type="module">import 'jest-browser-globals';</script>
    </head>
  </html>
`,
};
