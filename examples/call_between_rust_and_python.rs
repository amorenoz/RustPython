use rustpython::vm::{
    extend_module, pyclass, pymodule, builtins::PyDict, convert::ToPyObject, class::PyClassImpl, PyObjectRef, PyPayload, VirtualMachine,
};

pub fn main() {
    let interp = rustpython::InterpreterConfig::new()
        .init_stdlib()
        .init_hook(Box::new(|vm| {
            vm.add_native_module(
                "rust_py_module".to_owned(),
                Box::new(make_rust_module),
            );
        }))
        .interpreter();

    interp.enter(|vm| {
        vm.insert_sys_path(vm.new_pyobj("examples"))
            .expect("add path");

        let module = vm.import("call_between_rust_and_python", None, 0).unwrap_or_else(|excp| {vm.print_exception(excp); panic!()});

        // Call with payload
        let init_fn2 = module.get_attr("python_callback2", vm).unwrap_or_else(|excp| {vm.print_exception(excp); panic!()});

        let arg = other_module::RustStruct::new();
        init_fn2.call(vec!(arg.to_pyobject(vm)), vm).unwrap_or_else(|excp| {vm.print_exception(excp); panic!()});

        // Call with dict
        let arg = PyDict::new_ref(vm.as_ref());
        arg.set_item(&"rust_struct".to_string(),
                     other_module::RustStruct::new().to_pyobject(vm),
                     vm).expect("set_item");
        init_fn2.call(vec!(arg.to_pyobject(vm)), vm).unwrap_or_else(|excp| {vm.print_exception(excp); panic!()});
    })
}

fn make_rust_module(vm: &VirtualMachine) -> PyObjectRef {
    let module = rust_py_module::make_module(vm);
    let ctx = &vm.ctx;

    extend_module!(vm, module, {
        "RustStruct" => other_module::RustStruct::make_class(ctx),
    });
    module
}

#[pymodule]
mod rust_py_module {

}

mod other_module {
    use super::*;
    use rustpython::vm::{builtins::PyList, convert::ToPyObject, PyObjectRef};

    #[derive(Debug, Clone)]
    struct NumVec(Vec<i32>);

    impl ToPyObject for NumVec {
        fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
            let list = self.0.into_iter().map(|e| vm.new_pyobj(e)).collect();
            PyList::new_ref(list, vm.as_ref()).to_pyobject(vm)
        }
    }

    #[pyclass(module = false, name = "RustStruct")]
    #[derive(Debug, PyPayload)]
    pub struct RustStruct {
		#[allow(dead_code)]
        numbers: NumVec,
    }

    #[pyclass]
    impl RustStruct {
        pub fn new() -> RustStruct {
            RustStruct {
                numbers: NumVec(vec![1, 2, 3, 4]),
            }
        }
    }

}
