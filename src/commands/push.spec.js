const { push } = require("./push");
const { Timeslot } = require("../timeslot");
const { SpawnError, spawnAsync } = require("../spawnAsync");
const {
  getFirstPastCommitHash,
  getTrackingRemoteAndBranch,
  gitPush,
} = require("../git");
const { afterEach } = require("@jest/globals");

jest.mock("../git");

describe("push function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  test("returns error with empty timeslots config", async () => {
    expect(await push([], { getTimeslots: () => [] })).toBe(1);
  });
  test("returns error when pushing outside timeslots", async () => {
    const timeslotMock = {
      isDateWithin: jest.fn().mockReturnValueOnce(false),
    };
    expect(
      await push([], {
        getTimeslots: () => [timeslotMock],
        getAllowPushOutsideTimeslot: () => false,
      }),
    ).toBe(1);
  });
  test("pushes commits that are in the past", async () => {
    getFirstPastCommitHash.mockReturnValueOnce({
      commitHash: "ccbc6f931a3f3f88400c7159deee1e45b03cdd23",
      error: null,
    });
    getTrackingRemoteAndBranch.mockReturnValueOnce({
      remote: "origin",
      branch: "my-branch",
      error: null,
    });
    gitPush.mockReturnValueOnce(0);

    expect(
      await push([], {
        getTimeslots: () => [
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ],
        getAllowPushOutsideTimeslot: () => true,
      }),
    ).toBe(0);
    expect(gitPush.mock.calls[0]).toEqual([
      ["origin", "ccbc6f931a3f3f88400c7159deee1e45b03cdd23:my-branch"],
    ]);
  });
  test("fails gracefully when passing invalid git push options", async () => {
    getFirstPastCommitHash.mockReturnValueOnce({
      commitHash: "ccbc6f931a3f3f88400c7159deee1e45b03cdd23",
      error: null,
    });
    getTrackingRemoteAndBranch.mockReturnValueOnce({
      remote: "origin",
      branch: "my-branch",
      error: null,
    });
    gitPush.mockReturnValueOnce(1);

    expect(
      await push([], {
        getTimeslots: () => [
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ],
        getAllowPushOutsideTimeslot: () => true,
      }),
    ).toBe(1);
  });
  test("fails gracefully when there are no commits at all", async () => {
    getFirstPastCommitHash.mockReturnValueOnce({
      commitHash: null,
      error:
        "fatal: your current branch 'master' does not have any commits yet",
    });

    expect(
      await push([], {
        getTimeslots: () => [
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ],
        getAllowPushOutsideTimeslot: () => true,
      }),
    ).toBe(1);
  });
  test("fails gracefully when getting tracking branch fails", async () => {
    getFirstPastCommitHash.mockReturnValueOnce({
      commitHash: "ccbc6f931a3f3f88400c7159deee1e45b03cdd23",
      error: null,
    });
    getTrackingRemoteAndBranch.mockReturnValueOnce({
      remote: null,
      branch: null,
      error:
        "fatal: push destination 'refs/heads/master' on remote 'origin' has no local tracking branch",
    });

    expect(
      await push([], {
        getTimeslots: () => [
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ],
        getAllowPushOutsideTimeslot: () => true,
      }),
    ).toBe(1);
  });
  test("throws when spawnSync throws an unknown error", async () => {
    getFirstPastCommitHash.mockReturnValueOnce({
      commitHash: "ccbc6f931a3f3f88400c7159deee1e45b03cdd23",
      error: null,
    });
    getTrackingRemoteAndBranch.mockReturnValueOnce({
      remote: "origin",
      branch: "my-branch",
      error: null,
    });
    gitPush.mockReturnValueOnce(42);
    expect(
      await push([], {
        getTimeslots: () => [
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ],
        getAllowPushOutsideTimeslot: () => true,
      }),
    ).toEqual(42);
  });
});
