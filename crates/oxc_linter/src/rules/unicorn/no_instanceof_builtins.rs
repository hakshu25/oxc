use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_instanceof_builtins_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInstanceofBuiltins;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoInstanceofBuiltins,
    unicorn,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoInstanceofBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("fooExclude instanceof Function", Some(serde_json::json!([{"exclude": ["Function"]}]))),
        ("fooExclude instanceof Array", Some(serde_json::json!([{"exclude": ["Array"]}]))),
        ("fooExclude instanceof String", Some(serde_json::json!([{"exclude": ["String"]}]))),
        ("Array.isArray(arr)", None),
        ("arr instanceof array", None),
        ("a instanceof 'array'", None),
        ("a instanceof ArrayA", None),
        ("a.x[2] instanceof foo()", None),
        ("Array.isArray([1,2,3]) === true", None),
        (r#""arr instanceof Array""#, None),
    ];

    let fail = vec![
        ("fooInclude instanceof WebWorker", Some(serde_json::json!([{"include": ["WebWorker"]}]))),
        (
            "fooInclude instanceof HTMLElement",
            Some(serde_json::json!([{"include": ["HTMLElement"]}])),
        ),
        ("arr instanceof Array", None),
        ("[] instanceof Array", None),
        ("[1,2,3] instanceof Array === true", None),
        ("fun.call(1, 2, 3) instanceof Array", None),
        ("obj.arr instanceof Array", None),
        ("foo.bar[2] instanceof Array", None),
        ("(0, array) instanceof Array", None),
        ("function foo(){return[]instanceof Array}", None),
        (
            "(
				// comment
				((
					// comment
					(
						// comment
						foo
						// comment
					)
					// comment
				))
				// comment
			)
			// comment before instanceof\\r      instanceof
			// comment after instanceof
			(
				// comment
				(
					// comment
					Array
					// comment
				)
					// comment
			)
				// comment",
            None,
        ),
    ];

    Tester::new(NoInstanceofBuiltins::NAME, NoInstanceofBuiltins::PLUGIN, pass, fail)
        .test_and_snapshot();
}
