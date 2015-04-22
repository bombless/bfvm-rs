# TODO

Clever Generator
--
- [ ]
Implement `bf::Vm::print` as an instance of trait `bf::Echo`,
naming `NaiveMinimumMemory`, and implement 3 more instances,
`MinimumMemory`, `DumbSeek`, `NaiveShortestCode`

Quoted Value (Optional)
--
- [ ]
Add syntax element and `rt::Val` variant `Quote` to runtime,
representing a `rt:Val` that should not be evaluated,
using syntax ``(rt:Val content here)~`.
Clearly we should change old lambda syntax at the same time.
Consider using `#`, `%`, `^`, `&`, or `*` for leading character.
`+` won't be a choice here, because bf itself use `+` as element.
I prefer `~` for enclosing delim here

Evaluating Quoted Value (Optional)
--
- [ ]
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


The optional feature [Quoted Value](#user-content-quoted-value-optional)
will be our new lambda, just missing the ability to handle arguments.
We can introduce a new self-evaluating list handler that need to be placed in
lambda position to fully support lambda here.


----------------------
### New Ideas

- We can use lisp-like syntax here, use `'` to start a quoted value,
back-quote to start unquoting.
Then we need to remove old single-quote string literal syntax.
Starting from here, we can abandon old-style enclosing delim policy (or not).
The lambda from that optional feature,
[Quoted Value](#user-content-quoted-value-optional), works a la Common Lisp.
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
 break this limitation,

 * the other for values. The parser should track whether the value fetching
 is happened inside a function or not. If it is, that will be a new question
 for supporting function body. If not, we add it to global variable namespace.
- We may open a new repo, php-rs, for this last idea, I think.



# 计划清单

聪明的生成器
--
- [ ]
给`bf::Vm::print`实现成`bf::Echo`特征的一个实例，叫成`NaiveMinimumMemory`，
并且实现另外三个实例`MinimumMemory`、`DumbSeek`还有`NaiveShortestCode`

值引用 （可选）
--
- [ ]
增加语法元素以及`rt::Val`的一个变体`Quote`到运行时，
用来表示一个不被求值的`rt:Val`值，并且使用语法``(rt:Val 内容)~`。
显然，我们必须同时改变旧的函数语法。
考虑使用`#`、`%`、`^`、`&`，或者`*`来作为开始字符。
不该选`+`，因为 bf 已经有`+`了。
我倾向于用`~`作为结束用的分隔符。

对值引用求值（可选）
--
- [ ]
增加一个特性来求值值引用。不知道用什么语法好。

存储值
--
- [ ]
给`rt::Vm`特征增加一个新方法来存储`rt:Val`。也许
`fn store(&mut self, name: &str, rt::Val);`足够好了。语法方面我们可以用
`!<变量名>=<rt::Val 内容>~`。“=”号后面的内容不是可选的，并且它应该被即可求值。
我知道我们可以延迟求值以便绕开前面的可选的特性，不过那样看起来不自然。

把`macro_expand`重命名为`fetch`
--
- [ ]
意思是取回先前存储的`rt::Val`值。理论上取回的该是一个自求值类型比如字符串或空，
因为它被存起来的时候就应该已经被求值了。
我们先观望再看`Quote`型值在这个情况下是否该被求值。
考虑把语法换成`$名字~`。

那个选择性的特性
[值引用](#%E5%80%BC%E5%BC%95%E7%94%A8-%E5%8F%AF%E9%80%89)
会成为我们新的函数。
我们可以引进一个新的需要被放在函数位置的自求值表处理器来完全支持函数。


----------------------
### 新想法

- 我们可以使用类 Lisp 的语法，`'`来引用，反引号来解引用。
这样的话我们需要去掉旧的单引号字符串字面量语法。
从这里开始，我们需要丢掉旧的关闭分隔符策略（也可以不）。从
[值引用](#%E5%80%BC%E5%BC%95%E7%94%A8-%E5%8F%AF%E9%80%89)
这个选择性特性来的函数以 Common Lisp 的方式工作。
后面几项特性带有的自求值函数那一套函数以 Scheme 那样的方式工作。


- 我们可以以`val_left | val_right`还有`val_left & val_right`这样的语法，
增加逻辑或、逻辑与。内部表示将会是`Or(Rc<Val>, Rc<Val>)`等等。
我们可以用这种方式去掉`?`条件分支。

- 或者当新的分隔符政策到来，我们可以支持 PHP 风格的函数调用语法：
用标识符来调用虚拟机字节码，或者说，宏展开，用圆括号来做函数参数表
（就用普通的表，我们已经有了）。这样函数调用将仍然被称为是“在宏展开后运行”。
用这种方式，我们有了两种全局名字空间：
 * 一个是宏用的，惯例返回字节码，但是我们也可以去除这个限制，
 * 另一个是值的名字空间。解析器应该追踪值的获取是否发生在函数内。
 如果是的，就会引出关于函数体支持的问题。如果不是，我们把它添加到全局变量空间去。
- 我想我们也许要开一个新的仓库叫 php-rs，来支持最后这个想法。
