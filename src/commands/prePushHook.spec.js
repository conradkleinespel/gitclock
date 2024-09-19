const { prePushHook } = require("./prePushHook");
const { afterEach } = require("@jest/globals");
const { Timeslot } = require("../timeslot");
const fs = require("node:fs");
const { getPushObjectDate } = require("../git");
const { DateTime } = require("luxon");

jest.mock("node:fs");
jest.mock("../spawnAsync", () => {
  const originalModule = jest.requireActual("../spawnAsync");
  return {
    ...originalModule,
    spawnAsync: jest.fn(),
  };
});
jest.mock("../git");

describe("prePushHook function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  test("fails when there are no timeslots at all", async () => {
    const config = {
      getTimeslots: jest.fn().mockReturnValueOnce([]),
    };
    expect(await prePushHook(config)).toBe(1);
  });
  test("fails when there are no timeslots that match the current date", async () => {
    const currentDate = DateTime.now();
    const config = {
      getTimeslots: jest.fn().mockReturnValueOnce([
        new Timeslot(
          currentDate.weekday === 7 ? "1-1" : "7-7",
          "0900",
          "1700",
          "Africa/Nairobi",
        ), // only weekends
      ]),
      getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(false),
    };
    expect(await prePushHook(config)).toBe(1);
  });
  test("succeeds when there are no timeslots that match the current date but is allowed to push outside", async () => {
    fs.readFileSync.mockReturnValueOnce(
      `${"0".repeat(40)} ${"1".repeat(40)} refs/heads/master ${"2".repeat(40)}\n`,
    );

    getPushObjectDate.mockReturnValueOnce(
      DateTime.fromObject({ year: 2024, month: 4, day: 17, hour: 16 }),
    );

    const config = {
      getTimeslots: jest.fn().mockReturnValueOnce([
        new Timeslot("6-7", "0900", "1700", "Europe/Paris"), // only weekends
      ]),
      getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(true),
    };
    expect(await prePushHook(config)).toBe(0);
  });
  test("succeeds when there are no commits to push", async () => {
    fs.readFileSync.mockReturnValueOnce("");

    const config = {
      getTimeslots: jest.fn().mockReturnValueOnce([
        new Timeslot("6-7", "0900", "1700", "Europe/Paris"), // only weekends
      ]),
      getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(true),
    };
    expect(await prePushHook(config)).toBe(0);
  });
  test.each([
    [DateTime.now().minus({ hour: 1 }), 0],
    [DateTime.now().plus({ hour: 1 }), 1],
  ])(
    "succeeds when there are timeslots that match the current date, unless pushed commits are from the future",
    async (commitDate, expectedReturn) => {
      fs.readFileSync.mockReturnValueOnce(
        `${"0".repeat(40)} ${"1".repeat(40)} refs/heads/master ${"2".repeat(40)}\n`,
      );

      getPushObjectDate.mockReturnValueOnce(commitDate);

      const config = {
        getTimeslots: jest.fn().mockReturnValueOnce([
          new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"), // only weekends
        ]),
        getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(false),
      };
      expect(await prePushHook(config)).toBe(expectedReturn);
    },
  );
});
