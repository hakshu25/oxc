use oxc_ast::AstKind;
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
    OxcDiagnostic::warn("Disallow `instanceof` with built-in objects")
        .with_help("Avoid using `instanceof` for type checking as it can lead to unreliable results.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInstanceofBuiltins;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow instanceof with built-in objects.
    ///
    /// ### Why is this bad?
    ///
    /// Using instanceof to determine the type of an object has limitations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo instanceof String;
    /// foo instanceof Array;
    /// foo instanceof Object;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// typeof foo === 'string';
    /// Array.isArray(foo);
    /// Object.prototype.toString.call(foo) === '[object Object]';
    /// ```
    NoInstanceofBuiltins,
    unicorn,
    pedantic,
    conditional_fix_suggestion  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoInstanceofBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if matches!(node.kind(), AstKind::StringLiteral(_)) {
            return;
        }

        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };

        if !(expr.operator.is_instance_of() && expr.right.is_identifier_reference()) {
            return;
        }
        println!("expr: {expr:?}");

        let constructor_name = expr.right.get_identifier_reference().unwrap().name;
        println!("constructor_name: {constructor_name}");
        if constructor_name == "Array" {
            ctx.diagnostic_with_fix(no_instanceof_builtins_diagnostic(expr.span), |fixer| {
        let left_text = if expr.left.is_identifier_reference() {
            expr.left.get_identifier_reference().unwrap().name.to_string()
        } else {
            ctx.source_text()[expr.span].to_string()
        };

        fixer.replace(
            expr.span,
            format!("Array.isArray({})", left_text),
        )
        });
            return;
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ("fooExclude instanceof Function", Some(serde_json::json!([{"exclude": ["Function"]}]))),
        // ("fooExclude instanceof Array", Some(serde_json::json!([{"exclude": ["Array"]}]))),
        // ("fooExclude instanceof String", Some(serde_json::json!([{"exclude": ["String"]}]))),
        ("Array.isArray(arr)", None),
        ("arr instanceof array", None),
        ("a instanceof 'array'", None),
        ("a instanceof ArrayA", None),
        // ("a.x[2] instanceof foo()", None),
        ("Array.isArray([1,2,3]) === true", None),
        (r#""arr instanceof Array""#, None),
    ];

    let fail = vec![
        // ("fooInclude instanceof WebWorker", Some(serde_json::json!([{"include": ["WebWorker"]}]))),
        // (
        //     "fooInclude instanceof HTMLElement",
        //     Some(serde_json::json!([{"include": ["HTMLElement"]}])),
        // ),
        ("arr instanceof Array", None),
        ("[] instanceof Array", None),
        ("[1,2,3] instanceof Array === true", None),
        ("fun.call(1, 2, 3) instanceof Array", None),
        ("obj.arr instanceof Array", None),
        ("foo.bar[2] instanceof Array", None),
        ("(0, array) instanceof Array", None),
        ("function foo(){return[]instanceof Array}", None),
        // (
        //     "(
		// 		// comment
		// 		((
		// 			// comment
		// 			(
		// 				// comment
		// 				foo
		// 				// comment
		// 			)
		// 			// comment
		// 		))
		// 		// comment
		// 	)
		// 	// comment before instanceof\\r      instanceof
		// 	// comment after instanceof
		// 	(
		// 		// comment
		// 		(
		// 			// comment
		// 			Array
		// 			// comment
		// 		)
		// 			// comment
		// 	)
		// 		// comment",
        //     None,
        // ),
    ];

    let fix = vec![(
        "arr instanceof Array",
        "Array.isArray(arr)",
        None,
    ),
    (
        "[] instanceof Array",
        "Array.isArray([])",
        None,
    ),
    (
        "[1,2,3] instanceof Array === true",
        "Array.isArray([1,2,3]) === true",
        None,
    ),
    (
        "fun.call(1, 2, 3) instanceof Array",
        "Array.isArray(fun.call(1, 2, 3))",
        None,
    ),
    (
        "obj.arr instanceof Array",
        "Array.isArray(obj.arr)",
        None,
    ),
    (
        "foo.bar[2] instanceof Array",
        "Array.isArray(foo.bar[2])",
        None,
    ),
    (
        "(0, array) instanceof Array",
        "Array.isArray((0, array))",
        None,
    ),
    (
        "function foo(){return[]instanceof Array}",
        "function foo(){return Array.isArray([])}",
        None,
    )];

    Tester::new(NoInstanceofBuiltins::NAME, NoInstanceofBuiltins::PLUGIN, pass, fail)
        .expect_fix(fix).test_and_snapshot();
}
