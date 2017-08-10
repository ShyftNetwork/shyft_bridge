use web3::types::{Filter, FilterBuilder, Address, TransactionRequest, U256, H256, H160, Bytes, BlockNumber, Log};
use ethabi::{Contract, Token};
use error::{Error, ResultExt};

pub struct EthereumBridge {
	contract: Contract,
}

pub struct KovanBridge {
	contract: Contract,
}

pub struct KovanDeposit {
	recipient: Address,
	value: U256,
}

impl KovanBridge {
	pub fn deposit(&self, recipient: Address, value: U256, hash: H256) -> Bytes {
		let function = self.contract.function("deposit").expect("to find function `deposit`");
		let params = vec![
			Token::Address(recipient.0), 
			Token::Uint(value.0), 
			Token::FixedBytes(hash.0.to_vec())
		];
		let result = function.encode_call(params).expect("the params to be valid");
		Bytes(result)
	}

	pub fn deposits_filter(&self, address: Address, block: BlockNumber) -> Filter {
		let event = self.contract.event("Deposit").expect("to find event `Deposit`");
		FilterBuilder::default()
			.from_block(block.clone())
			.to_block(block)
			.address(vec![address])
			.topics(Some(vec![H256(event.signature())]), None, None, None)
			.build()
	}

	pub fn log_to_deposit(&self, log: Log) -> Result<KovanDeposit, Error> {
		let event = self.contract.event("Deposit").expect("to find event `Deposit`");
		let mut decoded = event.decode_log(
			log.topics.into_iter().map(|t| t.0).collect(),
			log.data.0
		)?;

		if decoded.len() != 2 {
			return Err("Invalid len of decoded deposit event".into())
		}

		let value = decoded.pop().and_then(|v| v.value.to_uint()).map(U256).chain_err(|| "expected uint")?;
		let recipient = decoded.pop().and_then(|v| v.value.to_address()).map(H160).chain_err(|| "expected address")?;

		let result = KovanDeposit {
			recipient,
			value,
		};

		Ok(result)
	}
}
