use browser_tester::Harness;

#[test]
fn issue_174_datetimeformat_accepts_chicago_and_formats_cross_zone_results()
-> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        function zonedText(instantMs, zone) {
          const formatter = new Intl.DateTimeFormat("en-US-u-nu-latn", {
            timeZone: zone,
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
            hour12: false,
          });
          const parts = formatter.formatToParts(new Date(instantMs));
          const get = (type) =>
            parts.find((part) => part.type === type)?.value || "?";
          return (
            get("year") +
            "-" +
            get("month") +
            "-" +
            get("day") +
            " " +
            get("hour") +
            ":" +
            get("minute") +
            ":" +
            get("second")
          );
        }

        const arrivalInstant = Date.UTC(2026, 0, 21, 8, 45, 0, 0);
        document.getElementById("out").textContent =
          zonedText(arrivalInstant, "America/Chicago") +
          "|" +
          zonedText(arrivalInstant, "America/New_York");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "2026-01-21 02:45:00|2026-01-21 03:45:00")?;
    Ok(())
}

#[test]
fn issue_175_datetimeformat_surfaces_new_york_dst_nonexistent_and_ambiguous_times()
-> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        function parseDateTimeParts(date, time) {
          const dateMatch = String(date || "").match(/^(\d{4})-(\d{2})-(\d{2})$/);
          const timeMatch = String(time || "").match(/^(\d{2}):(\d{2})$/);
          if (!dateMatch || !timeMatch) return null;
          return {
            year: Number(dateMatch[1]),
            month: Number(dateMatch[2]),
            day: Number(dateMatch[3]),
            hour: Number(timeMatch[1]),
            minute: Number(timeMatch[2]),
          };
        }

        function extractPartNumber(parts, type) {
          const value = parts.find((part) => part.type === type)?.value;
          const parsed = Number(value);
          return Number.isFinite(parsed) ? parsed : NaN;
        }

        function getZonedParts(epochMs, zone) {
          const formatter = new Intl.DateTimeFormat("en-US-u-nu-latn", {
            timeZone: zone,
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
            hour12: false,
          });
          const parts = formatter.formatToParts(new Date(epochMs));
          return {
            year: extractPartNumber(parts, "year"),
            month: extractPartNumber(parts, "month"),
            day: extractPartNumber(parts, "day"),
            hour: extractPartNumber(parts, "hour"),
            minute: extractPartNumber(parts, "minute"),
            second: extractPartNumber(parts, "second"),
          };
        }

        function getOffsetMinutes(zone, epochMs, knownParts) {
          const parts = knownParts || getZonedParts(epochMs, zone);
          const asUtcMs = Date.UTC(
            parts.year,
            parts.month - 1,
            parts.day,
            parts.hour,
            parts.minute,
            parts.second,
            0
          );
          const epochSecMs = Math.floor(epochMs / 1000) * 1000;
          return Math.round((asUtcMs - epochSecMs) / 60000);
        }

        function collectOffsetCandidates(zone, localUtcMs) {
          const offsets = new Set();
          [
            localUtcMs,
            localUtcMs - 3600000,
            localUtcMs + 3600000,
            localUtcMs - 86400000,
            localUtcMs + 86400000,
            localUtcMs - 172800000,
            localUtcMs + 172800000,
          ].forEach((sampleMs) => {
            const offset = getOffsetMinutes(zone, sampleMs);
            if (Number.isFinite(offset)) offsets.add(offset);
          });
          return Array.from(offsets);
        }

        function matchesLocalDateTime(parts, target) {
          return (
            parts.year === target.year &&
            parts.month === target.month &&
            parts.day === target.day &&
            parts.hour === target.hour &&
            parts.minute === target.minute
          );
        }

        function convertLocalToInstant(fromZone, date, time) {
          const parsed = parseDateTimeParts(date, time);
          const localUtcMs = Date.UTC(
            parsed.year,
            parsed.month - 1,
            parsed.day,
            parsed.hour,
            parsed.minute,
            0,
            0
          );
          const matches = [];
          collectOffsetCandidates(fromZone, localUtcMs).forEach((offsetMinutes) => {
            const candidateMs = localUtcMs - offsetMinutes * 60000;
            const candidateParts = getZonedParts(candidateMs, fromZone);
            if (!matchesLocalDateTime(candidateParts, parsed)) return;
            if (!matches.includes(candidateMs)) matches.push(candidateMs);
          });
          matches.sort((a, b) => a - b);
          if (!matches.length) return "error:nonexistent";
          if (matches.length === 1) return "ok";
          return "ambiguous";
        }

        document.getElementById("out").textContent =
          convertLocalToInstant("America/New_York", "2026-03-08", "02:30") +
          "|" +
          convertLocalToInstant("America/New_York", "2026-11-01", "01:30");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "error:nonexistent|ambiguous")?;
    Ok(())
}
