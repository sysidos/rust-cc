//! C types

use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq)]
pub struct Type
{
	pub basetype: BaseType,
	pub qualifiers: Qualifiers,
}

#[derive(PartialEq,Clone)]	/* Debug impl is manual */
pub enum BaseType
{
	Void,
	Bool,
	Struct(StructRef),
	Enum(EnumRef),
	Union(UnionRef),
	Float(FloatClass),
	Integer(IntClass),

	MagicType(MagicType),
	
	Pointer(Rc<Type>),
	Array(Rc<Type>, ArraySize),
	// TODO: make this a struct with a custom PartialEq impl that ignores names
	Function(FunctionType),
}
#[derive(Clone,PartialEq,Debug)]
pub enum MagicType
{
	VaList,
	Named(String, String),
}
#[derive(Clone,PartialEq)]
pub enum ArraySize
{
	None,
	Fixed(u64),
	Expr(ArraySizeExpr),
}
impl From<ArraySizeExpr> for ArraySize {
	fn from(v: ArraySizeExpr) -> Self {
		ArraySize::Expr(v)
	}
}
#[derive(Clone)]
pub struct ArraySizeExpr(Rc<::ast::Node>);
impl ArraySizeExpr {
	pub fn new(n: ::ast::Node) -> Self {
		ArraySizeExpr(Rc::new(n))
	}
}
impl PartialEq for ArraySizeExpr {
	fn eq(&self, v: &Self) -> bool {
		panic!("TODO: eq for ArraySizeExpr - {:?} == {:?}", self.0, v.0);
	}
}
impl ::std::ops::Deref for ArraySizeExpr {
	type Target = ::ast::Node;
	fn deref(&self) -> &::ast::Node {
		&*self.0
	}
}

#[derive(Clone,Debug)]
pub struct FunctionType
{
	pub ret: Rc<Type>,
	pub args: Vec<(Rc<Type>, String)>
}
impl PartialEq for FunctionType
{
	fn eq(&self, v: &Self) -> bool {
		self.ret == v.ret
			&& self.args.len() == v.args.len()
			// Checks just the base types (ignoring qualifiers like `const` on the top level)
			&& Iterator::zip( self.args.iter(), v.args.iter() ).all( |(a,b)| a.0.basetype == b.0.basetype )
	}
}

/// Boolean signedness
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Signedness
{
	Signed,
	Unsigned,
}
pub use self::Signedness::*;
impl Signedness {
	pub fn from_bool_signed(s: bool) -> Self {
		if s {
			Signedness::Signed
		}
		else {
			Signedness::Unsigned
		}
	}
	pub fn is_unsigned(&self) -> bool { *self == Signedness::Unsigned }
}
/// Qualifiers on a type (const, volatile, restrict)
// NOTE: `const volatile` is valid and has meaning (code can't change it, but hardware could)
#[derive(PartialEq,Clone)]
pub struct Qualifiers {
	v: u8,
}
impl Qualifiers {
	pub fn new() -> Self { Qualifiers { v: 0 } }

	pub fn set_const(&mut self) -> &mut Self { self.v |= 1; self }
	pub fn set_volatile(&mut self) -> &mut Self { self.v |= 2; self }
	pub fn set_restrict(&mut self) -> &mut Self { self.v |= 4; self }

	pub fn is_const(&self) -> bool { self.v & 1 != 0 }
	pub fn is_volatile(&self) -> bool { self.v & 2 != 0 }
	pub fn is_restrict(&self) -> bool { self.v & 4 != 0 }

	pub fn is_lesser_than(&self, other: &Self) -> bool {
		// If self is a subset of other (no missing bits
		self.v & other.v == self.v
	}

	pub fn merge_from(&mut self, other: &Qualifiers) {
		self.v |= other.v;
	}
}
impl ::std::fmt::Debug for Qualifiers {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}{}{}",
			if self.is_const() { "const " } else { "" },
			if self.is_volatile() { "volatile " } else { "" },
			if self.is_restrict() { "restrict " } else { "" },
			)
	}
}

/// Various integer types
#[derive(Debug,PartialEq,Clone)]
pub enum IntClass
{
	/// Fixed-size type
	Bits(Signedness,u8),
	/// `char` (three variants: char, signed char, and unsigned char)
	Char(Option<Signedness>),
	/// `[un]signed short [int]`
	Short(Signedness),
	/// `[un]signed int`
	Int(Signedness),
	/// `[un]signed long [int]`
	Long(Signedness),
	/// `[un]signed long long [int]`
	LongLong(Signedness),
}
impl IntClass {
	pub fn char() -> Self { IntClass::Char(None) }
	//pub fn uchar() -> Self { IntClass::Char(Some(Unsigned)) }
	//pub fn schar() -> Self { IntClass::Char(Some(Signed)) }
	pub const fn int() -> Self { IntClass::Int(Signed) }
}

#[derive(Debug,PartialEq,Clone)]
pub enum FloatClass
{
	Float,
	Double,
	LongDouble,
}

#[derive(Debug,PartialEq,Clone)]
pub enum StorageClass
{
	Auto,
	Extern,
	Static,
	Register,
}

pub type TypeRef = Rc<Type>;
pub type StructRef = RcRefCellPtrEq<Struct>;
pub type UnionRef  = RcRefCellPtrEq<Union>;
pub type EnumRef   = RcRefCellPtrEq<Enum>;

pub struct RcRefCellPtrEq<T>( Rc<RefCell<T>> );
impl<T> Clone for RcRefCellPtrEq<T> {
	fn clone(&self) -> Self {
		RcRefCellPtrEq(self.0.clone())
	}
}
impl<T> PartialEq for RcRefCellPtrEq<T> {
	fn eq(&self, x: &Self) -> bool {
		Rc::ptr_eq(&self.0, &x.0)
	}
}
impl<T> RcRefCellPtrEq<T> {
	pub fn new(v: T) -> Self {
		RcRefCellPtrEq( Rc::new(RefCell::new(v)) )
	}
	pub fn borrow(&self) -> ::std::cell::Ref<T> {
		self.0.borrow()
	}
	pub fn borrow_mut(&self) -> ::std::cell::RefMut<T> {
		self.0.borrow_mut()
	}
}

#[derive(Debug,PartialEq)]
pub struct Struct
{
	pub name: String,
	pub items: Option<Vec<(TypeRef,String)>>,
}

#[derive(Debug,PartialEq)]
pub struct Union
{
	pub name: String,
	items: Option<Vec<(TypeRef,String)>>,
}

#[derive(Debug,PartialEq)]
pub struct Enum
{
	pub name: String,
	items: Option<Vec<(u64,String)>>,
}

impl ::std::fmt::Debug for Type
{
	fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>
	{
		write!(fmt, "{:?}{:?}", self.qualifiers, self.basetype)
	}
}

impl ::std::fmt::Debug for BaseType
{
	fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>
	{
		match self
		{
		&BaseType::Void => write!(fmt, "void"),
		&BaseType::Bool => write!(fmt, "_Bool"),
		&BaseType::Struct(ref sr) => write!(fmt, "struct {:?}", sr.borrow().name),
		&BaseType::Union(ref ur)  => write!(fmt, "union {:?}",  ur.borrow().name),
		&BaseType::Enum(ref er)   => write!(fmt, "enum {:?}",   er.borrow().name),
		&BaseType::Float(ref fc) => write!(fmt, "{:?}", fc),
		&BaseType::Integer(ref ic) => write!(fmt, "{:?}", ic),
		&BaseType::MagicType(ref v) => write!(fmt, "/*magic*/ {:?}", v),
		
		&BaseType::Array(ref typeref, ref size) => write!(fmt, "{:?}{}", typeref, size),
		&BaseType::Pointer(ref typeref) => write!(fmt, "*{:?}", typeref),
		&BaseType::Function(ref info) => write!(fmt, "Fcn({:?}, {:?})", info.ret, info.args),
		}
	}
}

impl ::std::fmt::Display for ArraySize
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		match self
		{
		&ArraySize::None => f.write_str("[]"),
		&ArraySize::Fixed(v) => write!(f, "[{}]", v),
		&ArraySize::Expr(ref v) => write!(f, "[{:?}]", *v.0),
		}
	}
}

impl Type
{
	pub fn new_ref_bare(basetype: BaseType) -> TypeRef
	{
		Type::new_ref(basetype, Qualifiers::new())
	}
	pub fn new_ref(basetype: BaseType, qualifiers: Qualifiers) -> TypeRef
	{
		Rc::new(Type {
			basetype: basetype,
			qualifiers: qualifiers,
			})
	}
}

impl Struct
{
	pub fn new_ref(name: &str) -> StructRef
	{
		RcRefCellPtrEq::new(Struct {
			name: name.to_string(),
			items: None,
			})
	}
	
	pub fn is_populated(&self) -> bool
	{
		self.items.is_some()
	}
	pub fn set_items(&mut self, items: Vec<(TypeRef,String)>)
	{
		assert!( self.items.is_none() );
		self.items = Some(items);
	}
}

impl Union
{
	pub fn new_ref(name: &str) -> UnionRef
	{
		RcRefCellPtrEq::new(Union {
			name: name.to_string(),
			items: None,
			})
	}
	
	pub fn is_populated(&self) -> bool
	{
		self.items.is_some()
	}
	pub fn set_items(&mut self, items: Vec<(TypeRef,String)>)
	{
		assert!( self.items.is_none() );
		self.items = Some(items);
	}
}

impl Enum
{
	pub fn new_ref(name: &str) -> EnumRef
	{
		RcRefCellPtrEq::new(Enum {
			name: name.to_string(),
			items: None,
			})
	}
	
	pub fn is_populated(&self) -> bool
	{
		self.items.is_some()
	}
	pub fn set_items(&mut self, items: Vec<(u64,String)>)
	{
		assert!( self.items.is_none() );
		self.items = Some(items);
	}
}

// vim: ft=rust
