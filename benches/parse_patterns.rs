use criterion::{
  BatchSize, Criterion, black_box, criterion_group, criterion_main,
};
use url::Url;
use urlpattern::UrlPattern;
use urlpattern::UrlPatternInit;
use urlpattern::UrlPatternOptions;
use urlpattern::quirks::{self, EcmaRegexp};

/// Constructor strings covering the main pattern features: plain literals,
/// named groups, wildcards, optional/repeating modifiers and custom regexp
/// groups.
const CONSTRUCTOR_STRINGS: &[(&str, &str)] = &[
  ("literal", "/books"),
  ("named-group", "/books/:id"),
  ("wildcard", "/books/*"),
  ("optional-group", "/books/:id?"),
  ("repeating-group", "/books/:id+"),
  ("regexp-group", "/books/(\\d+)"),
  (
    "nested-groups",
    "/users/:user/posts/:post/comments/:comment",
  ),
  (
    "full-url",
    "https://:subdomain.example.com:8080/books/:id?q=:query",
  ),
  ("filename-wildcard", "component-ShippingGroupsSummary.*.js"),
];

/// The base URL used by the constructor string benchmarks. Parsed once, outside
/// of the measured sections.
const BASE_URL: &str = "https://example.test/web/";

fn bench_parse_shipping_groups_summary(c: &mut Criterion) {
  c.bench_function("parse component-ShippingGroupsSummary.*.js", |b| {
    b.iter(|| {
      let input = quirks::process_construct_pattern_input(
        black_box(quirks::StringOrInit::String(
          "component-ShippingGroupsSummary.*.js".into(),
        )),
        black_box(Some("https://example.test/web/")),
      );
      quirks::parse_pattern::<EcmaRegexp>(
        input.unwrap(),
        urlpattern::UrlPatternOptions::default(),
      )
      .unwrap();
    })
  });
}

/// Parse constructor strings into a [UrlPatternInit], without compiling the
/// components. This isolates the tokenizer and the constructor string parser.
fn bench_parse_constructor_string(c: &mut Criterion) {
  let base_url: Url = BASE_URL.parse().unwrap();
  let mut group = c.benchmark_group("parse_constructor_string");
  for (name, pattern) in CONSTRUCTOR_STRINGS {
    group.bench_function(*name, |b| {
      b.iter_batched(
        || base_url.clone(),
        |base_url| {
          UrlPatternInit::parse_constructor_string::<regex::Regex>(
            black_box(pattern),
            Some(base_url),
          )
          .unwrap()
        },
        BatchSize::SmallInput,
      )
    });
  }
  group.finish();
}

/// Full pattern compilation: constructor string parsing plus compiling every
/// component into its matcher and regexp.
fn bench_parse_pattern(c: &mut Criterion) {
  let base_url: Url = BASE_URL.parse().unwrap();
  let mut group = c.benchmark_group("parse_pattern");
  for (name, pattern) in CONSTRUCTOR_STRINGS {
    group.bench_function(*name, |b| {
      b.iter_batched(
        || base_url.clone(),
        |base_url| {
          let init = UrlPatternInit::parse_constructor_string::<regex::Regex>(
            black_box(pattern),
            Some(base_url),
          )
          .unwrap();
          <UrlPattern>::parse(init, UrlPatternOptions::default()).unwrap()
        },
        BatchSize::SmallInput,
      )
    });
  }
  group.finish();
}

/// Compile a pattern from a structured [UrlPatternInit] instead of a
/// constructor string.
fn bench_parse_init(c: &mut Criterion) {
  let pathname_init = UrlPatternInit {
    pathname: Some("/users/:id/posts/:post".to_owned()),
    ..Default::default()
  };
  c.bench_function("parse_init/pathname", |b| {
    b.iter_batched(
      || pathname_init.clone(),
      |init| <UrlPattern>::parse(init, UrlPatternOptions::default()).unwrap(),
      BatchSize::SmallInput,
    )
  });

  let all_components_init = UrlPatternInit {
    protocol: Some("http{s}?".to_owned()),
    username: Some(":user".to_owned()),
    password: Some(":password".to_owned()),
    hostname: Some(":subdomain.example.com".to_owned()),
    port: Some("8080".to_owned()),
    pathname: Some("/users/:id/posts/:post".to_owned()),
    search: Some("q=:query".to_owned()),
    hash: Some(":hash".to_owned()),
    ..Default::default()
  };
  c.bench_function("parse_init/all-components", |b| {
    b.iter_batched(
      || all_components_init.clone(),
      |init| <UrlPattern>::parse(init, UrlPatternOptions::default()).unwrap(),
      BatchSize::SmallInput,
    )
  });
}

/// The quirks entry points used by embedders (such as browsers), for both the
/// Rust regexp engine and the lazily evaluated ECMAScript one.
fn bench_quirks_parse_pattern(c: &mut Criterion) {
  let mut group = c.benchmark_group("quirks_parse_pattern");
  for (name, pattern) in CONSTRUCTOR_STRINGS {
    group.bench_function(format!("{name}/ecma"), |b| {
      b.iter(|| {
        let init = quirks::process_construct_pattern_input(
          black_box(quirks::StringOrInit::String((*pattern).into())),
          black_box(Some("https://example.test/web/")),
        )
        .unwrap();
        quirks::parse_pattern::<EcmaRegexp>(init, UrlPatternOptions::default())
          .unwrap()
      })
    });
    group.bench_function(format!("{name}/rust"), |b| {
      b.iter(|| {
        let init = quirks::process_construct_pattern_input(
          black_box(quirks::StringOrInit::String((*pattern).into())),
          black_box(Some("https://example.test/web/")),
        )
        .unwrap();
        quirks::parse_pattern::<regex::Regex>(
          init,
          UrlPatternOptions::default(),
        )
        .unwrap()
      })
    });
  }
  group.finish();
}

criterion_group!(
  benches,
  bench_parse_shipping_groups_summary,
  bench_parse_constructor_string,
  bench_parse_pattern,
  bench_parse_init,
  bench_quirks_parse_pattern
);
criterion_main!(benches);
