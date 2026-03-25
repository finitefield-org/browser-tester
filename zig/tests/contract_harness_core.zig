const std = @import("std");
const bt = @import("browser_tester_zig");

fn harnessFromHtml(html: []const u8) bt.Result(bt.Harness) {
    return bt.Harness.fromHtml(std.testing.allocator, html);
}

test "contract: stable core actions and assertions work together" {
    const html =
        \\<button id='run' type='button'>run</button>
        \\<form id='form'>
        \\  <input id='name' />
        \\  <input id='agree' type='checkbox' />
        \\  <button id='submitter' type='submit'>submit</button>
        \\</form>
        \\<p id='clicked'></p>
        \\<p id='submitted'></p>
        \\<script>
        \\  document.getElementById('run').addEventListener('click', () => {
        \\    document.getElementById('clicked').textContent = 'clicked';
        \\  });
        \\  document.getElementById('form').addEventListener('submit', (event) => {
        \\    event.preventDefault();
        \\    document.getElementById('submitted').textContent = [
        \\      document.getElementById('name').value,
        \\      String(document.getElementById('agree').checked),
        \\    ].join('|');
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.typeText("#name", "Alice");
    try subject.setChecked("#agree", true);
    try subject.click("#run");
    try subject.submit("#form");

    try subject.assertValue("#clicked", "clicked");
    try subject.assertValue("#submitted", "Alice|true");
    try subject.assertValue("#name", "Alice");
    try subject.assertChecked("#agree", true);
}

test "contract: Harness.assertExists reports presence and missing selectors" {
    var subject = try harnessFromHtml("<p id='present'>ok</p>");
    defer subject.deinit();

    try subject.assertExists("#present");
    try std.testing.expectError(bt.Error.AssertionFailed, subject.assertExists("#missing"));
}

test "contract: Harness constructors and time controls work" {
    const seeds = [_]bt.StorageSeed{
        .{
            .key = "token",
            .value = "seed",
        },
        .{
            .key = "mode",
            .value = "debug",
        },
    };
    const html =
        \\<p id='out'></p>
        \\<script>
        \\  const out = document.getElementById('out');
        \\  out.textContent = [
        \\    location.href,
        \\    localStorage.getItem('token'),
        \\    localStorage.getItem('mode'),
        \\  ].join('|');
        \\  setTimeout(() => {
        \\    out.textContent += '|done';
        \\  }, 25);
        \\</script>
    ;

    var subject = try bt.Harness.fromHtmlWithUrlAndLocalStorage(
        std.testing.allocator,
        "https://app.local/start",
        html,
        &seeds,
    );
    defer subject.deinit();

    try subject.assertValue("#out", "https://app.local/start|seed|debug");
    try subject.advanceTime(24);
    try subject.assertValue("#out", "https://app.local/start|seed|debug");
    try subject.advanceTime(1);
    try subject.assertValue("#out", "https://app.local/start|seed|debug|done");
}

test "contract: fetch mock is direct" {
    const html =
        \\<button id='run'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  document.getElementById('run').addEventListener('click', () => {
        \\    fetch('https://app.local/api/message')
        \\      .then((res) => res.text())
        \\      .then((text) => {
        \\        document.getElementById('out').textContent = text;
        \\      });
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.mocksMut().fetch().respondText("https://app.local/api/message", 200, "hello");
    try subject.click("#run");

    try subject.assertValue("#out", "hello");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fetch().calls().len);
    try std.testing.expectEqualStrings("https://app.local/api/message", subject.mocksMut().fetch().calls()[0].url);
}

test "contract: clipboard mock is direct" {
    var subject = try harnessFromHtml("<p id='out'></p>");
    defer subject.deinit();

    try subject.mocksMut().clipboard().seedText("seeded");

    try std.testing.expectEqualStrings("seeded", try subject.readClipboard());
    try subject.writeClipboard("copied");
    try std.testing.expectEqualStrings("copied", try subject.readClipboard());
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().clipboard().writes().len);
    try std.testing.expectEqualStrings("copied", subject.mocksMut().clipboard().writes()[0]);
}

test "contract: location navigation is recorded through the registry" {
    const html =
        \\<a id='go' href='https://app.local/next'>next</a>
    ;

    var subject = try bt.Harness.fromHtmlWithUrl(
        std.testing.allocator,
        "https://app.local/start",
        html,
    );
    defer subject.deinit();

    try subject.click("#go");
    try std.testing.expectEqualStrings("https://app.local/next", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://app.local/next", subject.mocksMut().location().navigations()[0]);
}

test "contract: file input selections are captured" {
    var subject = try harnessFromHtml("<input id='upload' type='file' multiple>");
    defer subject.deinit();

    try subject.setFiles("#upload", &.{ "first.txt", "second.txt" });

    try subject.assertValue("#upload", "first.txt, second.txt");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fileInput().selections().len);
    try std.testing.expectEqualStrings("#upload", subject.mocksMut().fileInput().selections()[0].selector);
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().fileInput().selections()[0].files.len);
    try std.testing.expectEqualStrings("first.txt", subject.mocksMut().fileInput().selections()[0].files[0]);
    try std.testing.expectEqualStrings("second.txt", subject.mocksMut().fileInput().selections()[0].files[1]);
}

test "contract: dialog and matchMedia mocks are direct" {
    const html =
        \\<button id='run'>run</button>
        \\<p id='out'></p>
        \\<script>
        \\  document.getElementById('run').addEventListener('click', () => {
        \\    const media = matchMedia('(min-width: 768px)');
        \\    const accepted = confirm('continue?');
        \\    const name = prompt('name?', 'guest');
        \\    alert('hello ' + name);
        \\    print();
        \\    document.getElementById('out').textContent = [
        \\      String(media.matches),
        \\      media.media,
        \\      String(accepted),
        \\      String(name),
        \\    ].join('|');
        \\  });
        \\</script>
    ;

    var subject = try harnessFromHtml(html);
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(min-width: 768px)", true);
    try subject.mocksMut().dialogs().pushConfirm(true);
    try subject.mocksMut().dialogs().pushPrompt("kazu");
    try subject.click("#run");

    try subject.assertValue("#out", "true|(min-width: 768px)|true|kazu");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings("(min-width: 768px)", subject.mocksMut().matchMedia().calls()[0].query);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().confirmMessages().len);
    try std.testing.expectEqualStrings("continue?", subject.mocksMut().dialogs().confirmMessages()[0]);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().promptMessages().len);
    try std.testing.expectEqualStrings("name?", subject.mocksMut().dialogs().promptMessages()[0]);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().alertMessages().len);
    try std.testing.expectEqualStrings("hello kazu", subject.mocksMut().dialogs().alertMessages()[0]);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
}
