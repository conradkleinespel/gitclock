const { spawn } = require("node:child_process");

class SpawnError extends Error {
  constructor(data) {
    super(data.message ?? "");
    this.code = data.code;
    this.stdout = data.stdout;
    this.stderr = data.stderr;
  }
}

async function spawnAsync(binary, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(binary, args, options);

    let stdout = "";
    let stderr = "";

    if (child.stdout) child.stdout.on("data", (data) => (stdout += data));
    if (child.stderr) child.stderr.on("data", (data) => (stderr += data));

    child.on("exit", (code, signal) => {
      if (code !== 0) {
        reject(new SpawnError({ code, stdout, stderr }));
      } else {
        resolve({ code, stdout, stderr });
      }
    });
  });
}

module.exports = {
  SpawnError,
  spawnAsync,
};
