const console = require("node:console");

exports.info = function (config) {
  console.log(`Config file: ${config.getFilePath()}`);
  return 0;
};
