const { preCommitHook } = require("./preCommitHook");
const { beforeEach, afterEach } = require("@jest/globals");
const { Timeslot } = require("../timeslot");
const { getLastCommitDate } = require("../git");
const { DateTime } = require("luxon");

jest.mock("../spawnAsync", () => {
  const originalModule = jest.requireActual("../spawnAsync");
  return {
    ...originalModule,
    spawnAsync: jest.fn(),
  };
});
jest.mock("../git");

describe("preCommitHook function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  describe("without GITCLOCK=1 in environment", () => {
    test("fails to prevent accidentally committing outside timeslot with raw git commit", async () => {
      const currentDate = DateTime.now();
      const config = {
        getTimeslots: jest
          .fn()
          .mockReturnValueOnce([
            new Timeslot(
              currentDate.weekday === 7 ? "1-1" : "7-7",
              "0900",
              "1200",
              "Africa/Nairobi",
            ),
          ]),
      };
      expect(await preCommitHook(config)).toBe(1);
    });
    test("fails to prevent accidentally committing not linear commit dates", async () => {
      const config = {
        getTimeslots: jest
          .fn()
          .mockReturnValueOnce([
            new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"),
          ]),
      };
      getLastCommitDate.mockReturnValueOnce(DateTime.now().plus({ day: 1 }));
      expect(await preCommitHook(config)).toBe(1);
    });
    test("succeeds when there are timeslots that match the current date", async () => {
      const config = {
        getTimeslots: jest.fn().mockReturnValueOnce([
          new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"), // only weekends
        ]),
        getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(false),
      };
      getLastCommitDate.mockReturnValueOnce(DateTime.now());

      expect(await preCommitHook(config)).toBe(0);
    });
  });

  describe("with GITCLOCK=1 in environment", () => {
    beforeEach(() => {
      process.env.GITCLOCK = "1";
    });
    afterEach(() => {
      delete process.env.GITCLOCK;
    });
    test("fails when there are no timeslots at all", async () => {
      const config = {
        getTimeslots: jest.fn().mockReturnValueOnce([]),
      };
      expect(await preCommitHook(config)).toBe(1);
    });
    test("succeeds when there are some timeslots to set the future commit date within", async () => {
      const config = {
        getTimeslots: jest.fn().mockReturnValueOnce([
          new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"), // only weekends
        ]),
      };
      getLastCommitDate.mockReturnValueOnce(DateTime.now().plus({ hour: 1 }));

      expect(await preCommitHook(config)).toBe(0);
    });
    test("succeeds when there are timeslots that match the current date", async () => {
      const config = {
        getTimeslots: jest.fn().mockReturnValueOnce([
          new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"), // only weekends
        ]),
        getAllowPushOutsideTimeslot: jest.fn().mockReturnValueOnce(false),
      };
      getLastCommitDate.mockReturnValueOnce(DateTime.now());

      expect(await preCommitHook(config)).toBe(0);
    });
  });
});
