function currentTime_ms(): number {
  return Date.now();
}

const DAYS = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

function msToString(ms: number, format: string) {
  const date = new Date(ms);

  format = format
    .replaceAll("{date}", date.getDate().toString())
    .replaceAll("{day}", DAYS[date.getDay()])
    .replaceAll("{mm}", (date.getMonth() + 1).toString().padStart(2, "0"))
    .replaceAll("{month}", MONTHS[date.getMonth()])
    .replaceAll("{yy}", date.getFullYear().toString().slice(-2))
    .replaceAll("{yyyy}", date.getFullYear().toString())
    .replaceAll("{hh24}", date.getHours().toString().padStart(2, "0"))
    .replaceAll(
      "{hh12}",
      hourTo12Hour(date.getHours()).toString().padStart(2, "0")
    )
    .replaceAll("{min}", date.getMinutes().toString().padStart(2, "0"))
    .replaceAll("{ss}", date.getSeconds().toString().padStart(2, "0"))
    .replaceAll("{ampm}", date.getHours() < 12 ? "am" : "pm");

  return format;
}

function hourTo12Hour(hour: number) {
  if (hour === 0 || hour === 12) {
    return 12;
  }

  return hour % 12;
}

const MINUTES_TO_SECONDS = 60;
const HOURS_TO_SECONDS = 60 * MINUTES_TO_SECONDS;
const DAYS_TO_SECONDS = 24 * HOURS_TO_SECONDS;

function relativeTimeToString(seconds: number): string {
  const days = Math.floor(seconds / DAYS_TO_SECONDS);
  seconds %= DAYS_TO_SECONDS;

  const hours = Math.floor(seconds / HOURS_TO_SECONDS);
  seconds %= HOURS_TO_SECONDS;

  const minutes = Math.floor(seconds / MINUTES_TO_SECONDS);
  seconds %= MINUTES_TO_SECONDS;

  seconds = Math.floor(seconds);

  let string = "";

  string += adjustForPluralAndZero(days, "day", true);
  string += adjustForPluralAndZero(hours, "hour", true);
  string += adjustForPluralAndZero(minutes, "minute", true);
  string += adjustForPluralAndZero(seconds, "second", false);

  return string === "" ? "0 seconds" : string;
}

function adjustForPluralAndZero(
  value: number,
  label: string,
  addSpace: boolean = false
): string {
  let string = "";

  if (value > 0) {
    string += `${value} ${label}`;

    if (value > 1) {
      string += "s";
    }

    if (addSpace) {
      string += " ";
    }
  }

  return string;
}

export { currentTime_ms, relativeTimeToString, msToString };
