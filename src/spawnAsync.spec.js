const { spawnAsync } = require("./spawnAsync");

describe("spawnAsync function", () => {
  test("throws SpawnError when exit code is not 0", async () => {
    const resultPromise = spawnAsync("ls", ["invalid"], {});
    expect(resultPromise).rejects.toThrow();

    const err = await resultPromise.catch((e) => e);

    expect(err.code).toBeGreaterThan(0);
    expect(err.stdout.length).toEqual(0);
    expect(err.stderr.length).toBeGreaterThan(0);
  });

  test("returns code and output when subprocess succeeds with stdio", async () => {
    const result = await spawnAsync("ls", ["/"], {});

    expect(result.code).toEqual(0);
    expect(result.stdout.length).toBeGreaterThan(0);
    expect(result.stderr.length).toEqual(0);
  });

  test("returns code without output when subprocess succeeds without stdio", async () => {
    const result = await spawnAsync("ls", ["/"], {
      stdio: "ignore",
    });

    expect(result.code).toEqual(0);
    expect(result.stdout.length).toEqual(0);
    expect(result.stderr.length).toEqual(0);
  });
});
