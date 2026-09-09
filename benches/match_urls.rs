use criterion::{
  BatchSize, Criterion, black_box, criterion_group, criterion_main,
};
use url::Url;
use urlpattern::UrlPattern;
use urlpattern::UrlPatternInit;
use urlpattern::UrlPatternMatchInput;
use urlpattern::UrlPatternOptions;
use urlpattern::quirks;

/// (name, pattern, matching url, non matching url)
const CASES: &[(&str, &str, &str, &str)] = &[
  (
    "literal",
    "https://example.test/books",
    "https://example.test/books",
    "https://example.test/movies",
  ),
  (
    "named-group",
    "https://example.test/books/:id",
    "https://example.test/books/123",
    "https://example.test/movies/123",
  ),
  (
    "wildcard",
    "https://example.test/books/*",
    "https://example.test/books/some/deeply/nested/path",
    "https://other.test/books/some/deeply/nested/path",
  ),
  (
    "regexp-group",
    "https://example.test/books/(\\d+)",
    "https://example.test/books/123",
    "https://example.test/books/abc",
  ),
  (
    "many-groups",
    "https://:sub.example.test/users/:user/posts/:post",
    "https://api.example.test/users/42/posts/1337",
    "https://api.example.test/users/42/comments/1337",
  ),
];

fn compile(pattern: &str) -> UrlPattern {
  let init = UrlPatternInit::parse_constructor_string::<regex::Regex>(
    pattern,
    Some("https://example.test/".parse().unwrap()),
  )
  .unwrap();
  <UrlPattern>::parse(init, UrlPatternOptions::default()).unwrap()
}

/// Execute a compiled pattern against a parsed URL, returning the matched
/// groups. The pattern and the URL are built outside of the measured section.
fn bench_exec(c: &mut Criterion) {
  let mut group = c.benchmark_group("exec");
  for (name, pattern, matching, non_matching) in CASES {
    let compiled = compile(pattern);
    let matching: Url = matching.parse().unwrap();
    let non_matching: Url = non_matching.parse().unwrap();

    group.bench_function(format!("{name}/match"), |b| {
      b.iter_batched(
        || matching.clone(),
        |url| {
          compiled
            .exec(UrlPatternMatchInput::Url(url))
            .unwrap()
            .unwrap()
        },
        BatchSize::SmallInput,
      )
    });
    group.bench_function(format!("{name}/no-match"), |b| {
      b.iter_batched(
        || non_matching.clone(),
        |url| {
          assert!(
            compiled
              .exec(UrlPatternMatchInput::Url(url))
              .unwrap()
              .is_none()
          )
        },
        BatchSize::SmallInput,
      )
    });
  }
  group.finish();
}

/// `test` only reports whether the URL matches, without building the result
/// object.
fn bench_test(c: &mut Criterion) {
  let mut group = c.benchmark_group("test");
  for (name, pattern, matching, _) in CASES {
    let compiled = compile(pattern);
    let matching: Url = matching.parse().unwrap();

    group.bench_function(*name, |b| {
      b.iter_batched(
        || matching.clone(),
        |url| assert!(compiled.test(UrlPatternMatchInput::Url(url)).unwrap()),
        BatchSize::SmallInput,
      )
    });
  }
  group.finish();
}

/// Matching against a structured [UrlPatternInit] instead of a parsed URL.
fn bench_exec_init(c: &mut Criterion) {
  let compiled = compile("https://example.test/users/:id");

  let init = UrlPatternInit {
    protocol: Some("https".to_owned()),
    hostname: Some("example.test".to_owned()),
    pathname: Some("/users/123".to_owned()),
    ..Default::default()
  };

  c.bench_function("exec_init/named-group", |b| {
    b.iter_batched(
      || init.clone(),
      |init| {
        compiled
          .exec(UrlPatternMatchInput::Init(init))
          .unwrap()
          .unwrap()
      },
      BatchSize::SmallInput,
    )
  });
}

/// The quirks match input pipeline used by embedders: turning a string (and an
/// optional base URL) into a canonicalized match input.
fn bench_quirks_match_input(c: &mut Criterion) {
  c.bench_function("quirks_process_match_input/relative", |b| {
    b.iter(|| {
      let (input, _) = quirks::process_match_input(
        black_box(quirks::StringOrInit::String("./books/123".into())),
        black_box(Some("https://example.test/web/")),
      )
      .unwrap()
      .unwrap();
      quirks::parse_match_input(input).unwrap()
    })
  });

  c.bench_function("quirks_process_match_input/absolute", |b| {
    b.iter(|| {
      let (input, _) = quirks::process_match_input(
        black_box(quirks::StringOrInit::String(
          "https://user:pass@example.test:8080/books/123?q=1#frag".into(),
        )),
        black_box(None),
      )
      .unwrap()
      .unwrap();
      quirks::parse_match_input(input).unwrap()
    })
  });
}

criterion_group!(
  benches,
  bench_exec,
  bench_test,
  bench_exec_init,
  bench_quirks_match_input
);
criterion_main!(benches);
