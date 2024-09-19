const { info } = require("./info");

describe("info function tests", () => {
  test("logs the config file path", () => {
    expect(info({ getFilePath: () => "/path/to/config" })).toBe(0);
  });
});
