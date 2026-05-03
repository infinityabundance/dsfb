# Phosphoric v0.1 Grammar

This document defines the initial source grammar for the Phosphoric v0.1 subset.

The grammar is intentionally small. Anything not listed here is unsupported and must be rejected by the parser rather than accepted as future-facing syntax.

## Lexical Categories

Identifiers:

- `Ident` starts with ASCII letter or `_`
- remaining characters may be ASCII letters, digits, or `_`

Literals:

- unsigned integer literals in base 10
- `true`
- `false`

Reserved keywords:

- `module`
- `capability`
- `struct`
- `enum`
- `fn`
- `effects`
- `let`
- `mut`
- `if`
- `else`
- `match`
- `for`
- `in`
- `return`
- `Result`
- `Option`
- `Some`
- `None`
- `Ok`
- `Err`

Punctuation:

- `(` `)` `{` `}` `[` `]`
- `:` `,` `;` `.`
- `..`
- `->`
- `=>`
- `=`

Operators in the initial profile:

- `+`
- `-`
- `*`
- `/`
- `==`
- `!=`
- `<`
- `<=`
- `>`
- `>=`

## Top-Level Grammar

```ebnf
Module        ::= "module" ModulePath ";" ProfileDecl? Item*
ModulePath    ::= Ident ("." Ident)*

ProfileDecl   ::= "profile" ProfileName ";"
ProfileName   ::= "boot" | "host" | "trusted" | "runtime"

Item          ::= CapabilityDecl
                | StructDecl
                | EnumDecl
                | FunctionDecl

CapabilityDecl ::= "capability" Ident ";"

StructDecl    ::= "struct" Ident "{" FieldList? "}"
FieldList      ::= Field ("," Field)* ","?
Field          ::= Ident ":" Type

EnumDecl      ::= "enum" Ident "{" VariantList? "}"
VariantList    ::= Variant ("," Variant)* ","?
Variant        ::= Ident
                | Ident "(" TypeList? ")"

FunctionDecl  ::= "fn" Ident "(" ParamList? ")" ReturnType? EffectDecl? Block
ParamList      ::= Param ("," Param)* ","?
Param          ::= Ident ":" Type
ReturnType     ::= "->" Type
EffectDecl     ::= "effects" "(" EffectList? ")"
EffectList     ::= Ident ("," Ident)*
```

## Type Grammar

```ebnf
Type          ::= PrimitiveType
                | PathType
                | ArrayType
                | SliceType
                | ResultType
                | OptionType

PrimitiveType ::= "u8" | "u16" | "u32" | "u64"
                | "i8" | "i16" | "i32" | "i64"
                | "bool"

PathType      ::= Ident ("." Ident)*
ArrayType     ::= "[" Type ";" IntegerLiteral "]"
SliceType     ::= "Slice" "[" Type "," IntegerLiteral "]"
ResultType    ::= "Result" "[" Type "," Type "]"
OptionType    ::= "Option" "[" Type "]"

TypeList      ::= Type ("," Type)* ","?
```

## Statement Grammar

```ebnf
Block         ::= "{" Statement* Expr? "}"

Statement     ::= LetStmt
                | AssignStmt
                | ExprStmt
                | ForStmt
                | ReturnStmt

LetStmt       ::= "let" Mutability? Ident TypeAnn? "=" Expr ";"
Mutability    ::= "mut"
TypeAnn       ::= ":" Type
AssignStmt    ::= Place "=" Expr ";"
ExprStmt      ::= Expr ";"
ForStmt       ::= BoundAttr "for" Ident "in" RangeExpr Block
BoundAttr     ::= "#[" "bound" "=" BoundExpr "]"
BoundExpr     ::= IntegerLiteral | Ident
ReturnStmt    ::= "return" Expr? ";"

RangeExpr     ::= Expr ".." Expr
```

## Expression Grammar

```ebnf
Expr          ::= IfExpr
                | MatchExpr
                | BinaryExpr

IfExpr        ::= "if" Expr Block ("else" Block)?
MatchExpr     ::= "match" Expr "{" MatchArm+ "}"
MatchArm      ::= Pattern "=>" MatchBody ","?
MatchBody     ::= Expr | Block

BinaryExpr    ::= CallExpr (BinaryOp CallExpr)*
BinaryOp      ::= "+" | "-" | "*" | "/"
                | "==" | "!=" | "<" | "<=" | ">" | ">="

CallExpr      ::= PostfixExpr ("(" ArgList? ")")*
ArgList       ::= Expr ("," Expr)* ","?

PostfixExpr   ::= PrimaryExpr (FieldAccess)*
FieldAccess   ::= "." Ident

PrimaryExpr   ::= IntegerLiteral
                | "true"
                | "false"
                | UnitExpr
                | PathExpr
                | TupleLikeExpr
                | ArrayExpr
                | Block
                | "(" Expr ")"

UnitExpr      ::= "(" ")"

PathExpr      ::= Ident ("." Ident)*
TupleLikeExpr ::= PathExpr "(" ArgList? ")"
ArrayExpr     ::= "[" ArrayElements? "]"
ArrayElements ::= Expr ("," Expr)* ","?

Place         ::= Ident ("." Ident)* PlaceIndex?
PlaceIndex    ::= "[" Expr "]"
```

## Pattern Grammar

```ebnf
Pattern       ::= "_"
                | IntegerLiteral
                | "true"
                | "false"
                | Ident
                | VariantPattern

VariantPattern ::= PathExpr
                 | PathExpr "(" PatternList? ")"

PatternList    ::= Pattern ("," Pattern)* ","?
```

## Block Semantics

A `Block` is `"{" Statement* Expr? "}"`. The optional trailing `Expr` is the block's value. When a block is the body of a function and the function declares a non-unit return type, the trailing `Expr` is the function's return value — equivalent to wrapping it in an explicit `return Expr;`. A block with no trailing `Expr` has type `()` (unit) and may only be used where the unit type is acceptable.

This semantics is not a future addition — it is the meaning the grammar already implies. It is recorded explicitly here so that auditors do not have to infer it from `Block`'s shape.

## Unit Type and Unit Expression

The unit type `()` and the unit expression `( )` are part of the grammar. The unit type is the type of:

- a function declared without a `ReturnType` clause
- a `Block` with no trailing `Expr`
- a `match` arm whose body is `()` or a unit-typed block
- the unit expression itself

Unit is *not* a stand-in for "no value." It is a concrete zero-sized value with one inhabitant. Type checking treats it like any other type. The unit expression is permitted everywhere a value-producing expression is permitted, but compiler diagnostics may reject pointless unit constructions (see code `K-017`).

## Current Guarantees

This grammar currently guarantees:

- modules, data declarations, and functions are the only top-level constructs
- `match` is the only pattern-based control structure
- arrays and bounded slices are first-class grammar forms
- `Result[...]` and `Option[...]` are named syntactic forms in the subset
- effect annotations are attached directly to function declarations
- the unit type `()` is part of the type system; the unit expression `( )` is a `PrimaryExpr`
- a `Block`'s trailing `Expr` is the block's value (and the function's return value when the block is a function body)
- `Place` (the LHS of `AssignStmt`) accepts an optional trailing array index, making `arr[i] = expr;` legal for fixed-array and bounded-slice targets

## Unsupported And Rejected Syntax

The parser must reject syntax for:

- macros
- traits
- impl blocks
- classes
- inheritance
- `async`
- closures
- `while`
- unrestricted `loop`
- `unsafe`
- borrow operators such as `&` and `&mut`
- pointer syntax
- generic parameter lists
- variadic functions
- string literals
- floating-point literals and types

## Future Work That Is Not Yet Promised

The following syntax families are intentionally absent:

- const generics beyond fixed literal capacities
- richer path-qualified imports
- pattern guards
- methods and receiver syntax
- trait bounds or interface declarations

These remain outside the grammar until a later document adds them explicitly.
