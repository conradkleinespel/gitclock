const { afterEach } = require("@jest/globals");
const {
  rewriteHistory,
  chooseMinDateForNewCommit,
} = require("./rewriteHistory");
const { Timeslot } = require("../timeslot");
const {
  getLogShaAndDates,
  LogEntry,
  resetHard,
  cherryPick,
  amendWithNewDate,
} = require("../git");
const { getNextCommitDate } = require("./commit");
const { DateTime } = require("luxon");

jest.mock("../git");
jest.mock("./commit");

describe("rewrite-history function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  test("returns 1 when rewriting history without timeslots", async () => {
    const result = await rewriteHistory({
      getTimeslots: jest.fn().mockReturnValueOnce([]),
    });

    expect(result).toBe(1);
  });

  test("cherry picks all commits rewriting the dates within timeslots", async () => {
    resetHard.mockReturnValueOnce(null);
    cherryPick.mockReturnValue(null);
    getNextCommitDate.mockReturnValue(new Date());
    amendWithNewDate.mockReturnValue(null);

    getLogShaAndDates.mockReturnValueOnce([
      new LogEntry("a".repeat(40), DateTime.now(), DateTime.now()),
      new LogEntry("b".repeat(40), DateTime.now(), DateTime.now()),
      new LogEntry("c".repeat(40), DateTime.now(), DateTime.now()),
    ]);

    const result = await rewriteHistory({
      getTimeslots: jest
        .fn()
        .mockReturnValueOnce([
          new Timeslot("1-7", "0000", "2359", "Europe/Paris"),
        ]),
      getTimezone: jest.fn().mockReturnValue(null),
    });

    expect(result).toBe(0);
    expect(resetHard.mock.calls.length).toBe(1);
    expect(cherryPick.mock.calls.length).toBe(2);
    expect(amendWithNewDate.mock.calls.length).toBe(3);
  });

  test.each([
    [
      new Date("2023-07-04 10:00:00 +0300"),
      new Date("2023-07-05 10:00:00 +0300"),
      null,
      new Date("2023-07-05 10:00:00 +0300"),
    ],
    [
      new Date("2023-07-05 10:00:00 +0300"),
      new Date("2023-07-04 10:00:00 +0300"),
      null,
      new Date("2023-07-05 10:00:00 +0300"),
    ],
    [
      new Date("2023-07-04 10:00:00 +0300"),
      new Date("2023-07-05 10:00:00 +0300"),
      new Date("2023-07-06 10:00:00 +0300"),
      new Date("2023-07-06 10:00:00 +0300"),
    ],
    [
      new Date("2023-07-04 10:00:00 +0300"),
      new Date("2023-07-05 10:00:00 +0300"),
      new Date("2023-07-03 10:00:00 +0300"),
      new Date("2023-07-05 10:00:00 +0300"),
    ],
  ])(
    "chooseMinDateForNewCommit() picks the latest date to prevent out of order commits",
    async (
      existingAuthorDate,
      existingCommitDate,
      lastCommitDate,
      expected,
    ) => {
      expect(
        chooseMinDateForNewCommit(
          existingAuthorDate,
          existingCommitDate,
          lastCommitDate,
        ),
      ).toEqual(expected);
    },
  );
});
