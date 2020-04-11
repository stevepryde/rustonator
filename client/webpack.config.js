const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
require("webpack");
require("dotenv").config();

const phaserModulePath = path.join(__dirname, "/node_modules/phaser-ce/");

const clientConfig = {
  mode: "development",
  entry: "./src/js/client.ts",
  module: {
    rules: [
      {test: /pixi\.js/, loader: "expose-loader?PIXI"},
      {test: /phaser-split\.js$/, loader: "expose-loader?Phaser"},
      {test: /p2\.js/, loader: "expose-loader?p2"},
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node-modules/
      }
    ]
  },
  resolve: {
    alias: {
      "phaser-ce": path.join(phaserModulePath, "build/custom/phaser-split.js"),
      pixi: path.join(phaserModulePath, "build/custom/pixi.js"),
      p2: path.join(phaserModulePath, "build/custom/p2.js")
    },
    extensions: [".tsx", ".ts", ".js"]
  },
  target: "web",
  output: {
    path: path.join(__dirname, "dist", "client"),
    filename: "app.js"
  },
  plugins: [
    new CleanWebpackPlugin({cleanOnceBeforeBuildPatterns: ["dist"]}),
    new HtmlWebpackPlugin({template: "./src/index.html"}),
    new CopyWebpackPlugin([
      {from: "src/*.html", ignore: ["index.html"], flatten: true},
      {from: "src/css", to: "css/"},
      {from: "src/assets", to: "assets/"},
      {from: "src/*.ico", flatten: true},
      {from: "src/*.png", flatten: true}
    ])
  ]
};

module.exports = [clientConfig];
