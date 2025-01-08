mod actions;
mod http_action;
mod promise;
mod vm;

pub use actions::*;
pub use http_action::*;
pub use promise::{Promise, PromiseStatus};
pub use vm::*;

#[path = ""]
#[cfg(test)]
mod test {
    use arbitrary::{Arbitrary, Unstructured};
    use rand::RngCore;

    use super::*;

    #[macro_export]
    macro_rules! test_ser_deser {
				($($type:ty),*) => {
						$(
							::paste::paste! {
								#[test]
								fn [<test_ser_deser_$type:snake>]() {
										let mut rng = rand::thread_rng();
										let mut bytes = Vec::with_capacity(1024);
										rng.fill_bytes(&mut bytes);
										let mut data = Unstructured::new(&bytes);
										let value = <$type>::arbitrary(&mut data).unwrap();
										let ser = serde_json::to_string(&value).unwrap();
										let deser = serde_json::from_str::<$type>(&ser).unwrap();
										assert_eq!(value, deser);
								}
							}
						)*
				};
		}

    test_ser_deser!(
        CallSelfAction,
        ChainSendTxAction,
        ChainTxStatusAction,
        ChainViewAction,
        ConsensusType,
        DatabaseGetAction,
        DatabaseSetAction,
        ExecutionResult,
        ExitInfo,
        HttpFetchAction,
        HttpFetchMethod,
        HttpFetchOptions,
        HttpFetchResponse,
        MainChainCallAction,
        MainChainQueryAction,
        MainChainViewAction,
        Promise,
        PromiseAction,
        PromiseStatus,
        TriggerEventAction,
        VmCallData,
        VmResult,
        VmResultStatus,
        VmType,
        WasmId
    );
}
