const typescript = require("@rollup/plugin-typescript");
const pkg = require("./package.json");

export default [
  {
    input: "src/index_es.ts",
    output: [
      { file: pkg.module, format: "es" },
    ],
    plugins: [typescript()],
  },
  {
    input: "src/index_cjs_node.ts",
    output: [
      { file: pkg.main, format: "cjs" },
    ],
    plugins: [typescript()],
  },
];
