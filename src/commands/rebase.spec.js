const { rebase } = require("./rebase");
const { afterEach } = require("@jest/globals");
const { Timeslot } = require("../timeslot");
const { gitRebase } = require("../git");

jest.mock("../git");

describe("rebase function tests", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  test("returns error with empty timeslots config", async () => {
    expect(await rebase([], { getTimeslots: () => [] })).toBe(1);
  });

  test("runs rebase when config is valid", async () => {
    gitRebase.mockReturnValue(0);
    expect(
      await rebase(["--some", "option"], {
        getTimeslots: jest
          .fn()
          .mockReturnValueOnce([
            new Timeslot("1-7", "0000", "2359", "Africa/Nairobi"),
          ]),
      }),
    ).toBe(0);
    expect(gitRebase.mock.calls[0]).toEqual([
      ["--committer-date-is-author-date", "--some", "option"],
    ]);
  });
});
