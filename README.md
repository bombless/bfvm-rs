# TODO

Clever Generator
--
- [ ]
Implement `bf::Vm::print` as an instance of trait `bf::Echo`,
naming `NaiveMinimumMemory`, and implement 3 more instances,
`MinimumMemory`, `DumbSeek`, `NaiveShortestCode`

Quoted Value
--
- [ ] (Optional)
Add syntax element and `rt::Val` variant `Quote` to runtime,
representing a `rt:Val` that should not be evaluated,
using syntax ``(rt:Val content here)~`.
Clearly we should change old lambda syntax at the same time.
Consider using `#`, `%`, `^`, `&`, or `*` for leading character.
`+` won't be a choice here, because bf itself use `+` as element.
I prefer `~` for enclosing delim here

Evaluating Quoted Value
--
- [ ] (Optional)
Add feature to evaluate quoted value.
Have no idea what syntax should be used here

Storing Value
--
- [ ]
Add a new method to `rt::Vm` trait to store `rt:Val`.
maybe `fn store(&mut self, name: &str, rt::Val);` is good enough.
As for the syntax, we can use `!<variablename>=<rt::Val content>~`.
The rt:Val value is not optional, and it is evaluated immediately.
I know we can lazy-evaluate it to bypass former optional feature,
but that will not look natural

Rename `macro_expand` to `fetch`
--
- [ ]
It means fetching earlier stored `rt::Val` value.
In theory it should be a self-evaluated value, like string or nil,
because it should be evaluated when it was stored to VM earlier.
We'll see if we need to evaluate `Quote` value for this case.
Consider to change the syntax to `$name~`.
Also we need to rename the method, maybe
`fn fetch(&mut self, name: &str)->rt::Val;`


The optional feature [Quoted Value](#quoted-value) will be our new lambda,
just missing the ability to handle arguments.
We can introduce a new self-evaluating list handler that need to be placed in
lambda position to fully support lambda here.


----------------------
### New Ideas

- We can use lisp-like syntax here, use `'` to start a quoted value, `` ` to
start unquoting.
Then we need to remove old single-quote string literal syntax.
Starting from here, we can abandon old-style enclosing delim policy (or not).
The lambda from that optional feature, [Quoted Value](#quoted-value), works
a la Common Lisp.
Self-evaluating lambdas that later entries suggest work a la Scheme.

- We can add "logic or" as well as "logic and",
using `val_left | val_right` and `val_left & val_right` syntax.
The representation will be `Or(Rc<Val>, Rc<Val>)`, etc.
We can then remove the `?` conditional branch in this manner.

- Or if the new enclosing delim policy is landed, we can support a PHP-style
function calling syntax: plain identifier to call VM byte code, or say,
expanding macro, round brackets for arguments list (plain list here, we already
have it). So function call will still be called run-after-macro-expanding.
In this manner, we have 2 global namespaces:
 * one for macros, in convention we return byte code value here, but we may
 break this limitation.

 * the other for values. The parser should track whether the value fetching
 is happened inside a function or not. If it is, that will be a new question
 for supporting function body. If not, we add it to global variable namespace.
- We may open a new repo, php-rs, for this last idea, I think.
