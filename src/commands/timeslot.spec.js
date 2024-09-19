const { timeslot } = require("./timeslot");
const { Timeslot } = require("../timeslot");

describe("timeslot function tests", () => {
  test("incompatible options throw", () => {
    expect(timeslot({ add: true, list: true }, {})).toEqual(1);
    expect(timeslot({ add: true }, {})).toEqual(1);
    expect(timeslot({}, {})).toEqual(1);
  });

  test.each([
    ["", "", ""],
    ["invalid", "0900", "1700"],
  ])("returns error with invalid input", (days, start, end) => {
    const configMock = {
      addTimeslot: jest.fn().mockImplementationOnce((days, start, end) => {
        new Timeslot(days, start, end);
      }),
      getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
    };
    expect(timeslot({ add: true, days, start, end }, configMock)).toBe(1);
  });

  test("--add adds timeslot to config when input is valid", () => {
    let configDict = {
      timeslots: [{ days: "6-7", start: "0900", end: "2300" }],
    };

    const configMock = {
      addTimeslot: jest
        .fn()
        .mockImplementationOnce((days, start, end) =>
          configDict["timeslots"].push({ days, start, end }),
        ),
      getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
    };

    expect(
      timeslot(
        { add: true, days: "1-5", start: "0900", end: "1700" },
        configMock,
      ),
    ).toBe(0);
    expect(configDict.timeslots).toEqual([
      { days: "6-7", start: "0900", end: "2300" },
      { days: "1-5", start: "0900", end: "1700" },
    ]);
  });

  test("--list returns 0 for empty list", () => {
    expect(
      timeslot(
        { list: true },
        {
          getTimeslots: jest.fn().mockReturnValueOnce([]),
          getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
        },
      ),
    ).toBe(0);
  });

  test("--list returns 0 for full list", () => {
    expect(
      timeslot(
        { list: true },
        {
          getTimeslots: jest
            .fn()
            .mockReturnValueOnce([
              new Timeslot("1-5", "0900", "1700", "Europe/Paris"),
            ]),
          getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
        },
      ),
    ).toBe(0);
  });
});
