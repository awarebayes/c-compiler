use crate::{
    common::{StorageClass, Width},
    ir::nodes,
};

pub trait IrTextRepr {
    fn to_ir_string(&self) -> String;
}

impl IrTextRepr for Width {
    fn to_ir_string(&self) -> String {
        match self {
            Self::Byte => "b".into(),
            Self::Long => "l".into(),
            Self::Short => "s".into(),
            Self::Word => "w".into(),
        }
    }
}

impl IrTextRepr for nodes::Op {
    fn to_ir_string(&self) -> String {
        match self {
            Self::Plus => "+".into(),
            Self::Mul => "*".into(),
            Self::Minus => "-".into(),
            Self::Div => "/".into(),
            Self::Gt => ">".into(),
            Self::Lt => "<".into(),
            Self::Eq => "==".into(),
        }
    }
}
impl IrTextRepr for nodes::AddressConstant {
    fn to_ir_string(&self) -> String {
        match self {
            Self::Numeric(n) => format!("#{n}"),
            Self::StringLiteral(n) => format!("s'{n}'"),
        }
    }
}

impl IrTextRepr for nodes::Address {
    fn to_ir_string(&self) -> String {
        match self {
            Self::CompilerTemp(ct) => format!("%_t{}", ct),
            Self::Source((s, c)) => format!("%{}.{}", s, c),
            Self::Constant(c) => c.to_ir_string(),
        }
    }
}

impl IrTextRepr for nodes::Label {
    fn to_ir_string(&self) -> String {
        match self {
            Self::CompilerTemp(ct) => format!("_l{}", ct),
            Self::Source(s) => format!("{}", s),
        }
    }
}

impl IrTextRepr for nodes::Quadriplet {
    fn to_ir_string(&self) -> String {
        match self.right.as_ref() {
            Some(right) => format!(
                "{} ={} {} {} {}",
                self.dest.to_ir_string(),
                self.width.to_ir_string(),
                self.left.to_ir_string(),
                self.op.to_ir_string(),
                right.to_ir_string()
            ),
            None => format!(
                "%{} ={} {} %{}",
                self.dest.to_ir_string(),
                self.width.to_ir_string(),
                self.op.to_ir_string(),
                self.left.to_ir_string()
            ),
        }
    }
}

impl IrTextRepr for nodes::Ssa {
    fn to_ir_string(&self) -> String {
        match self {
            nodes::Ssa::Quadriplet(quadriplet) =>format!("\t{}", quadriplet.to_ir_string()),
            nodes::Ssa::Return { value } => match value.as_ref() {
                Some((addr, width)) => {
                    format!("\treturn {} {}", width.to_ir_string(), addr.to_ir_string())
                }
                None => format!("\treturn"),
            },
            nodes::Ssa::Assignment {
                dest,
                source,
                width,
            } => {
                // TODO: lvalue cannot be constant
                format!(
                    "\t{} ={} {}",
                    dest.to_ir_string(),
                    width.to_ir_string(),
                    source.to_ir_string()
                )
            }
            nodes::Ssa::Param { value, width, number } => {
                format!("\tparam{} {} {}", number, width.to_ir_string(), value.to_ir_string())
            }
            nodes::Ssa::Call {
                dest,
                func,
                num_params: _,
            } => match dest {
                Some((addr, width)) => {
                    format!(
                        "\t{} ={} call {}",
                        addr.to_ir_string(),
                        width.to_ir_string(),
                        func.to_ir_string()
                    )
                }
                None => {
                    format!("\tcall {}", func.to_ir_string())
                }
            },
            nodes::Ssa::Label(label) => {
                format!("@{}:", label.to_ir_string())
            }
            nodes::Ssa::Branch {
                cond,
                true_target,
                false_target,
            } => {
                format!(
                    "\tbranch {}: {} {}",
                    cond.to_ir_string(),
                    true_target.to_ir_string(),
                    false_target.to_ir_string()
                )
            }
            nodes::Ssa::Jump(label) => {
                format!("\tjump {}", label.to_ir_string())
            },
            nodes::Ssa::Phi(phi) => {
                let merging_expr = phi.merging.iter().map(| (addr, lab) | {
                    format!("[{}, @{}]", addr.to_ir_string(), lab.to_ir_string())
                }).collect::<Vec<_>>().join(", ");
                format!("\t{} ={} phi {}", phi.dest.to_ir_string(), phi.width.to_ir_string(), merging_expr)
            }
        }
    }
}

impl IrTextRepr for StorageClass {
    fn to_ir_string(&self) -> String {
        match self {
            Self::Auto => "auto".into(),
            Self::Extern => "extern".into(),
        }
    }
}

impl IrTextRepr for nodes::ToplevelItem {
    fn to_ir_string(&self) -> String {
        match self {
            nodes::ToplevelItem::Function(f) => {
                let parameters: Vec<String> = f
                    .parameters
                    .iter()
                    .map(|(n, w)| format!("{} %{}", w.to_ir_string(), n))
                    .collect();
                let parameters = parameters.join(", ");

                let mut s = format!(
                    "function {} {} ({}) {{\n",
                    f.return_width.to_ir_string(),
                    f.name,
                    parameters
                );
                for block in &f.body {
                    s.push_str(&block.to_ir_string());
                    s.push_str("\n");
                }
                s.push_str("}\n");
                s
            }
            nodes::ToplevelItem::Declaration(d) => match d {
                nodes::ToplevelDeclaration::Function {
                    storage_class,
                    name,
                    return_width,
                    parameters,
                } => {
                    let parameters: Vec<String> =
                        parameters.iter().map(|w| w.to_ir_string()).collect();
                    let parameters = parameters.join(", ");
                    format!(
                        "{} ${name} = \"{name}\": ({parameters}) -> {}",
                        storage_class.to_ir_string(),
                        return_width.to_ir_string()
                    )
                }
            },
        }
    }
}

pub fn into_text(input: &[nodes::ToplevelItem]) -> String {
    let mut res = String::new();
    for item in input {
        res.push_str(&item.to_ir_string());
        res.push('\n');
    }
    res
}
