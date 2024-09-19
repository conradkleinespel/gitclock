const { commit, getNextCommitDate } = require("./commit");
const { SpawnError, spawnAsync } = require("../spawnAsync");
const { Timeslot } = require("../timeslot");
const { afterEach } = require("@jest/globals");
const { getLastCommitDate, gitCommit } = require("../git");
const { DateTime } = require("luxon");

// TODO

jest.mock("../git");

describe("commit function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  describe("commit", () => {
    test.each([
      [
        DateTime.now().minus({ day: 1 }),
        DateTime.now(),
        DateTime.now().plus({ minute: 15 }),
      ],
      [
        DateTime.now().plus({ day: 1 }),
        DateTime.now().plus({ day: 1 }),
        DateTime.now().plus({ day: 1, minute: 15 }),
      ],
    ])(
      "commit returns 0 when everything is valid",
      async (
        lastCommitDate,
        expectedNextCommitDateMin,
        expectedNextCommitDateMax,
      ) => {
        getLastCommitDate.mockReturnValueOnce(lastCommitDate);
        gitCommit.mockReturnValue(0);

        const args = ["-m", "My commit message"];
        const result = await commit(args, {
          getTimeslots: jest
            .fn()
            .mockReturnValueOnce([
              new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"),
            ]),
          getTimezone: jest.fn().mockReturnValue("Africa/Nairobi"),
        });

        expect(result).toBe(0);
        expect(gitCommit).toBeCalledTimes(1);
        const nextCommitDate = gitCommit.mock.calls[0][0];
        expect(nextCommitDate >= expectedNextCommitDateMin).toBeTruthy();
        expect(nextCommitDate <= expectedNextCommitDateMax).toBeTruthy();
        expect(gitCommit.mock.calls[0].slice(1)).toEqual([
          "Africa/Nairobi",
          args,
        ]);
      },
    );
    test("returns 1 when committing without timeslots", async () => {
      const result = await commit(["-m", "My commit message"], {
        getTimeslots: jest.fn().mockReturnValueOnce([]),
      });

      expect(result).toBe(1);
    });
    test("returns 1 when git commit fails", async () => {
      getLastCommitDate.mockReturnValueOnce(
        DateTime.fromObject({ year: 2023, month: 7, day: 4, hour: 10 }),
      );
      gitCommit.mockReturnValueOnce(1);
      const args = ["-m", "My commit message"];
      const result = await commit(args, {
        getTimeslots: jest
          .fn()
          .mockReturnValueOnce([
            new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"),
          ]),
        getTimezone: jest.fn().mockReturnValue("Africa/Nairobi"),
      });

      expect(result).toBe(1);
      expect(gitCommit).toBeCalledTimes(1);
    });
  });

  describe("getNextCommitDate", () => {
    test.each([
      [
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 0,
        }),
        [new Timeslot("1-7", "0000", "2359", "Africa/Nairobi")],
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 0,
        }),
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 17,
        }),
      ],
      [
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 0,
        }),
        [new Timeslot("1-7", "1000", "1600", "Africa/Nairobi")],
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 0,
        }),
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 17,
        }),
      ],
      [
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 0,
        }),
        [
          new Timeslot("6-7", "0100", "2359", "Africa/Nairobi"),
          new Timeslot("1-5", "1000", "1600", "Africa/Nairobi"),
        ],
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 0,
        }),
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 17,
        }),
      ],
      [
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 9,
          minute: 0,
        }),
        [
          new Timeslot("6-7", "0100", "2359", "Africa/Nairobi"),
          new Timeslot("6-7", "0200", "2359", "Africa/Nairobi"),
          new Timeslot("1-5", "1000", "1600", "Africa/Nairobi"),
        ],
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 0,
        }),
        DateTime.fromObject({
          year: 2023,
          month: 7,
          day: 4,
          hour: 10,
          minute: 17,
        }),
      ],
    ])(
      "getNextCommitDate function returns a time within the timeslots",
      (minDate, timeslots, minExpectedDate, maxExpectedDate) => {
        const result = getNextCommitDate(minDate, minDate, timeslots);

        console.log(
          [
            `[${timeslots.map((s) => `(${s})`).join(", ")}]`,
            minExpectedDate,
            maxExpectedDate,
            `=> ${result}`,
          ].join("\n"),
        );

        expect(result).toBeInstanceOf(DateTime);
        expect(result.toUnixInteger()).toBeGreaterThanOrEqual(
          minExpectedDate.toUnixInteger(),
        );
        expect(result.toUnixInteger()).toBeLessThanOrEqual(
          maxExpectedDate.toUnixInteger(),
        );
      },
    );
  });
});
