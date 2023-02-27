use cairo_lang_sierra::program::GenericArg::Value;
use cairo_lang_sierra::program::LibfuncDeclaration;
use inkwell::debug_info::{AsDIScope, DIFlags, DIFlagsConstants, DISubprogram};
use inkwell::types::{BasicType, StringRadix};

use crate::sierra::errors::DEBUG_NAME_EXPECTED;
use crate::sierra::llvm_compiler::Compiler;

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    /// Implementation of the LLVM IR conversion of a felt constant.
    ///
    /// # Arguments
    ///
    /// * `libfunc_declaration` - The corelib function declaration of felt_const.
    ///
    /// # Error
    ///
    /// Panics if the felt type has not been declared previously.
    pub fn felt_const(&self, libfunc_declaration: &LibfuncDeclaration) {
        // We could hardcode the LLVM IR type for felt but this adds a check.
        let return_type = self.get_type_from_name("felt").expect("Can't get felt from name");
        // fn felt_const() -> felt
        let func = self.module.add_function(
            libfunc_declaration.id.debug_name.clone().expect(DEBUG_NAME_EXPECTED).to_string().as_str(),
            return_type.fn_type(&[], false),
            None,
        );
        self.builder.position_at_end(self.context.append_basic_block(func, "entry"));

        // Convert the bigint value of the constant into an LLVM IR const int value. Panics if the constant
        // value is not a decimal value.
        let ret = if let Value(val) = &libfunc_declaration.long_id.generic_args[0] {
            return_type
                .as_basic_type_enum()
                .into_int_type()
                .const_int_from_string(val.to_string().as_str(), StringRadix::Decimal)
                .expect("Couldn't convert to string the felt constant value")
        } else {
            // If the constant doesn't have any value it panics because a constant should have a value.
            panic!("No value for felt constant")
        };
        self.builder.build_return(Some(&ret));

        if let Some(dibuilder) = &self.dibuilder {
            let ditypes = self.ditypes.as_ref().unwrap();
            let compile_unit = self.compile_unit.unwrap();
            let return_type = ditypes.get(self.get_type_id_from_name("felt").unwrap()).unwrap();
            let subroutine_type =
                dibuilder.create_subroutine_type(compile_unit.get_file(), Some(*return_type), &[], DIFlags::PUBLIC);
            let func_scope: DISubprogram<'_> = dibuilder.create_function(
                compile_unit.as_debug_info_scope(),
                libfunc_declaration.id.debug_name.clone().unwrap().as_str(),
                None,
                compile_unit.get_file(),
                self.current_line_estimate, // line number
                subroutine_type,
                true,
                true,
                self.current_line_estimate, // scope line
                DIFlags::PUBLIC,
                false,
            );
            func.set_subprogram(func_scope);
        }
    }
}
