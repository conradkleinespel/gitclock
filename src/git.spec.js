const {
  getLastCommitDate,
  getTrackingRemoteAndBranch,
  getFirstPastCommitHash,
  getPushObjectDate,
  LogEntry,
  getLogShaAndDates,
  cherryPick,
  resetHard,
  amendWithNewDate,
  gitCommit,
  gitPush,
  gitRebase,
} = require("./git");
const { SpawnError, spawnAsync } = require("./spawnAsync");
const { afterEach } = require("@jest/globals");
const { DateTime } = require("luxon");

jest.mock("./spawnAsync", () => {
  const originalModule = jest.requireActual("./spawnAsync");
  return {
    ...originalModule,
    spawnAsync: jest.fn(),
  };
});

describe("git function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  describe("getPushObjectDate", () => {
    test("returns a Date instance from the output date", async () => {
      const dateString = "2023-07-04T10:00:00.000+02:00";
      const date = DateTime.fromISO("2023-07-04T10:00:00.000+02:00");
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: `${dateString}\n`,
        stderr: "",
      });
      expect((await getPushObjectDate()).toMillis()).toEqual(date.toMillis());
    });
    test("throws any error as-is", async () => {
      const err = new SpawnError({
        code: 1,
        stdout: "",
        stderr: "some git error",
      });
      spawnAsync.mockReturnValueOnce(Promise.reject(err));
      expect(getPushObjectDate()).rejects.toThrow(err);
    });
  });

  describe("getLastCommitDate", () => {
    test("throws error if it is not a SpawnError", async () => {
      const err = new Error("unknown error");
      spawnAsync.mockReturnValueOnce(Promise.reject(err));
      expect(getLastCommitDate()).rejects.toThrow(err);
    });
    test("calls spawnAsync and returns the last commit date when there is one", async () => {
      const dateString = "2023-07-04T10:17:00.000+02:00";
      const date = DateTime.fromISO(dateString);
      spawnAsync.mockReturnValueOnce(
        Promise.resolve({
          code: 0,
          stdout: dateString,
          stderr: "",
        }),
      );
      const result = await getLastCommitDate();
      expect(result.toMillis()).toEqual(date.toMillis());
    });

    test("calls spawnAsync and returns current date when there is no commit", async () => {
      const date = DateTime.now();
      spawnAsync.mockReturnValueOnce(
        Promise.reject(new SpawnError({ code: 42, stdout: "", stderr: "" })),
      );
      const result = await getLastCommitDate();
      expect(result < date.plus({ minute: 1 })).toBeTruthy();
      expect(result > date.minus({ minute: 1 })).toBeTruthy();
    });
  });
  describe("getTrackingRemoteAndBranch", () => {
    test("throws error if it is not a SpawnError", async () => {
      const err = new Error("unknown error");
      spawnAsync.mockReturnValueOnce(Promise.reject(err));
      expect(getTrackingRemoteAndBranch()).rejects.toThrow(err);
    });
    test("returns a git error reformatted", async () => {
      spawnAsync.mockReturnValueOnce(
        Promise.reject(
          new SpawnError({
            code: 1,
            stdout: "",
            stderr: "some git error",
          }),
        ),
      );
      const result = await getTrackingRemoteAndBranch();
      expect(result).toEqual({
        remote: null,
        branch: null,
        error: "some git error",
      });
    });
    test("returns the remote and branch", async () => {
      spawnAsync.mockReturnValueOnce(
        Promise.resolve({
          code: 1,
          stdout: "origin/master",
          stderr: "",
        }),
      );
      const result = await getTrackingRemoteAndBranch();
      expect(result).toEqual({
        remote: "origin",
        branch: "master",
        error: null,
      });
    });
  });
  describe("getFirstPastCommitHash", () => {
    test("throws error if it is not a SpawnError", async () => {
      const err = new Error("unknown error");
      spawnAsync.mockReturnValueOnce(Promise.reject(err));
      expect(getFirstPastCommitHash()).rejects.toThrow(err);
    });
    test("returns a git error reformatted", async () => {
      spawnAsync.mockReturnValueOnce(
        Promise.reject(
          new SpawnError({
            code: 1,
            stdout: "",
            stderr: "some git error",
          }),
        ),
      );
      const result = await getFirstPastCommitHash();
      expect(result).toEqual({ commitHash: null, error: "some git error" });
    });
    test("returns latest commit hash", async () => {
      spawnAsync.mockReturnValueOnce(
        Promise.resolve({
          code: 1,
          stdout: `${"a".repeat(40)}\n`,
          stderr: "",
        }),
      );
      const result = await getFirstPastCommitHash();
      expect(result).toEqual({ commitHash: "a".repeat(40), error: null });
    });
  });

  describe("LogEntry tests", () => {
    test("data is stored correctly", async () => {
      const entry = new LogEntry(
        "abcd",
        DateTime.now().plus({ day: 1 }),
        DateTime.now().plus({ day: 2 }),
      );

      expect(entry.sha).toEqual("abcd");
      expect(entry.authorDate.day).toEqual(DateTime.now().plus({ day: 1 }).day);
      expect(entry.commitDate.day).toEqual(DateTime.now().plus({ day: 2 }).day);
    });
  });

  describe("getLogShaAndDates", () => {
    test("returns a list of log entries", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: `
        ${"a".repeat(40)} 2023-07-01T10:00:00.000Z 2023-07-02T10:00:00.000Z
        ${"b".repeat(40)} 2023-07-03T10:00:00.000Z 2023-07-04T10:00:00.000Z
        `,
        stderr: "",
      });

      const logEntries = await getLogShaAndDates();
      expect(logEntries.map((l) => l.sha)).toEqual([
        "a".repeat(40),
        "b".repeat(40),
      ]);
      expect(logEntries.map((l) => l.authorDate.day)).toEqual([1, 3]);
      expect(logEntries.map((l) => l.commitDate.day)).toEqual([2, 4]);
    });
  });

  describe("cherryPick", () => {
    test("calls git cherry-pick with the passed sha", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });

      await cherryPick("a".repeat(40));
      expect(spawnAsync).toBeCalledTimes(1);
      expect(spawnAsync.mock.calls[0][0]).toEqual("git");
      expect(spawnAsync.mock.calls[0][1]).toEqual([
        "cherry-pick",
        "a".repeat(40),
      ]);
    });
  });

  describe("resetHard", () => {
    test("calls git reset --hard with the passed sha", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });

      await resetHard("a".repeat(40));
      expect(spawnAsync).toBeCalledTimes(1);
      expect(spawnAsync.mock.calls[0][0]).toEqual("git");
      expect(spawnAsync.mock.calls[0][1]).toEqual([
        "reset",
        "--hard",
        "a".repeat(40),
      ]);
    });
  });

  describe("amendWithNewDate", () => {
    test("amends and sets date in env and --date", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });

      const newDate = DateTime.fromObject({
        year: 2023,
        month: 8,
        day: 4,
        hour: 10,
      });
      await amendWithNewDate(newDate, null);
      expect(spawnAsync).toBeCalledTimes(1);
      expect(spawnAsync.mock.calls[0][0]).toEqual("git");
      expect(spawnAsync.mock.calls[0][1]).toEqual([
        "commit",
        "--amend",
        "--no-edit",
        "--date",
        `${newDate.toUnixInteger()} +0300`,
      ]);
      expect(spawnAsync.mock.calls[0][2].env.GIT_AUTHOR_DATE).toEqual(
        `${newDate.toUnixInteger()} +0300`,
      );
      expect(spawnAsync.mock.calls[0][2].env.GIT_COMMITTER_DATE).toEqual(
        `${newDate.toUnixInteger()} +0300`,
      );
    });
  });

  describe("gitCommit", () => {
    test("throws when spawnSync throws an unknown error", async () => {
      spawnAsync.mockReturnValueOnce(
        Promise.reject(new Error("unknown error")),
      );
      const args = ["-m", "My commit message"];
      expect(gitCommit(DateTime.now(), "Europe/Paris", args)).rejects.toThrow(
        "unknown error",
      );
    });
    test("returns error code when spawnSync throws a SpawnError", async () => {
      const err = new SpawnError({
        code: 42,
        stdout: "",
        stderr: "",
      });
      spawnAsync.mockReturnValueOnce(Promise.reject(err));
      const args = ["-m", "My commit message"];
      expect(await gitCommit(DateTime.now(), "Europe/Paris", args)).toEqual(42);
    });
    test("returns 0 when spawnSync runs without error", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });
      const args = ["-m", "My commit message"];
      expect(await gitCommit(DateTime.now(), "Europe/Paris", args)).toEqual(0);
    });
  });

  describe("gitPush", () => {
    test("calls git push with the passed args", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });

      await gitPush(["--some", "option"]);
      expect(spawnAsync).toBeCalledTimes(1);
      expect(spawnAsync.mock.calls[0][0]).toEqual("git");
      expect(spawnAsync.mock.calls[0][1]).toEqual(["push", "--some", "option"]);
    });
  });

  describe("gitRebase", () => {
    test("calls git rebase with the passed args", async () => {
      spawnAsync.mockReturnValueOnce({
        code: 0,
        stdout: "",
        stderr: "",
      });

      await gitRebase(["--some", "option"]);
      expect(spawnAsync).toBeCalledTimes(1);
      expect(spawnAsync.mock.calls[0][0]).toEqual("git");
      expect(spawnAsync.mock.calls[0][1]).toEqual([
        "rebase",
        "--some",
        "option",
      ]);
    });
  });
});
