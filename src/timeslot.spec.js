const { Timeslot } = require("./timeslot");
const { DateTime } = require("luxon");

describe("Timeslot Class", () => {
  describe("constructor", () => {
    test("throws error on invalid day range", () => {
      expect(() => new Timeslot("8-9", "0900", "1500")).toThrow(
        "Invalid day range format. It should be like 1-7",
      );
      expect(() => new Timeslot("2-1", "0900", "1500")).toThrow(
        "Invalid day range, first day should be lower or equal to second day",
      );
    });
    test.each([
      [undefined, "0900", "1700"],
      ["1-5", undefined, "1700"],
      ["1-5", "0900", undefined],
    ])(
      "throws error when passing undefined configuration",
      (days, start, end) => {
        expect(() => new Timeslot(days, start, end)).toThrow(
          `Invalid timeslot [days=${days}, start=${start}, end=${end}]`,
        );
      },
    );

    test("throws error on invalid start time", () => {
      expect(() => new Timeslot("1-5", "2500", "1500")).toThrow(
        "Invalid start time format, it should be like 0900 or 1530",
      );
    });

    test("throws error on invalid end time", () => {
      expect(() => new Timeslot("1-5", "0900", "2500")).toThrow(
        "Invalid end time format, it should be like 0900 or 1530",
      );
    });
  });

  describe("isDateWithin", () => {
    test.each([
      [20, 16, 0, "Asia/Kolkata", false], // Saturday
      [17, 16, 0, "Asia/Kolkata", false], // Wednesday, late hour
      [17, 15, 1, "Asia/Kolkata", false], // Wednesday, late minute
      [17, 8, 0, "Asia/Kolkata", false], // Wednesday, early hour
      [17, 9, 10, "Asia/Kolkata", false], // Wednesday, early minute
      [17, 9, 45, "Asia/Kolkata", true], // Wednesday, within schedule
      [17, 14, 45, "Africa/Nairobi", false], // Wednesday, within schedule but wrong timezone so outside schedule
    ])(
      "returns the right value depending on start and end time",
      (day, hour, minute, computerTimezone, expected) => {
        const timeslot = new Timeslot("1-5", "0915", "1500", "Asia/Kolkata");
        const date = DateTime.fromObject(
          {
            year: 2024,
            month: 4,
            day,
            hour,
            minute,
          },
          { zone: computerTimezone },
        );
        expect(timeslot.isDateWithin(date)).toBe(expected);
      },
    );
  });

  describe("nextSuitableDate", () => {
    test.each([
      ["Africa/Nairobi", 9, 0],
      ["Asia/Kolkata", 6, 30],
    ])(
      "returns next day if time exceeds end time",
      (timezone, expectedHour, expectedMinute) => {
        const timeslot = new Timeslot("1-5", "0900", "1500", timezone);
        const wednesdayAtFourPM = DateTime.fromObject({
          year: 2024,
          month: 4,
          day: 17,
          hour: 16,
        }); // April 17, 2024, 16:00 is a Wednesday
        const result = timeslot.nextSuitableDate(wednesdayAtFourPM);
        expect(result.day).toBe(18);
        expect(result.hour).toBe(expectedHour);
        expect(result.minute).toBe(expectedMinute);
      },
    );

    test.each([
      ["Africa/Nairobi", 9, 0],
      ["Asia/Kolkata", 6, 30],
    ])(
      "returns first day of next week if date is in weekend",
      (timezone, expectedHour, expectedMinute) => {
        const timeslot = new Timeslot("1-5", "0900", "1500", timezone);
        const result = timeslot.nextSuitableDate(
          DateTime.fromObject({ year: 2024, month: 4, day: 21 }),
        ); // April 21, 2024 is a Sunday
        expect(result.day).toBe(22);
        expect(result.hour).toBe(expectedHour);
        expect(result.minute).toBe(expectedMinute);
      },
    );

    test.each([
      ["Africa/Nairobi", 9, 0],
      ["Asia/Kolkata", 6, 30],
    ])(
      "returns date itself if it is within",
      (timezone, expectedHour, expectedMinute) => {
        const timeslot = new Timeslot("1-5", "0900", "1500", timezone);
        const monday = DateTime.fromObject({
          year: 2024,
          month: 4,
          day: 21,
          hour: 10,
          minute: 45,
        }); // April 21, 2024 is a Sunday
        const result = timeslot.nextSuitableDate(monday);
        expect(result.day).toBe(22);
        expect(result.hour).toBe(expectedHour);
        expect(result.minute).toBe(expectedMinute);
      },
    );
  });

  describe("Timeslot toString tests", () => {
    test("returns correct string for Monday to Friday", () => {
      const timeslot = new Timeslot("1-5", "0900", "1500", "Africa/Nairobi");
      const result = timeslot.toString();
      expect(result).toBe(
        "Monday to Friday, between 09:00 and 15:00 in timezone Africa/Nairobi",
      );
    });

    test("returns correct string for Saturday only", () => {
      const timeslot = new Timeslot("6-6", "1000", "1400", "Africa/Nairobi");
      const result = timeslot.toString();
      expect(result).toBe(
        "Saturday, between 10:00 and 14:00 in timezone Africa/Nairobi",
      );
    });
  });
});
