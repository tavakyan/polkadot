// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Network packet message types. These get serialized and put into the lower level protocol payload.

use runtime_primitives::traits::{Block as BlockT, Header as HeaderT};
use codec::{Encode, Decode, Input, Output};
pub use self::generic::{BlockAnnounce, RemoteCallRequest, ConsensusVote, SignedConsensusVote, FromBlock};

/// A unique ID of a request.
pub type RequestId = u64;

/// Type alias for using the message type using block type parameters.
pub type Message<B> = generic::Message<
	B,
	<B as BlockT>::Header,
	<B as BlockT>::Hash,
	<<B as BlockT>::Header as HeaderT>::Number,
	<B as BlockT>::Extrinsic,
>;

/// Type alias for using the status type using block type parameters.
pub type Status<B> = generic::Status<
	<B as BlockT>::Hash,
	<<B as BlockT>::Header as HeaderT>::Number,
>;

/// Type alias for using the block request type using block type parameters.
pub type BlockRequest<B> = generic::BlockRequest<
	<B as BlockT>::Hash,
	<<B as BlockT>::Header as HeaderT>::Number,
>;

/// Type alias for using the localized bft message type using block type parameters.
pub type LocalizedBftMessage<B> = generic::LocalizedBftMessage<
	B,
	<B as BlockT>::Hash,
>;

/// Type alias for using the BlockData type using block type parameters.
pub type BlockData<B> = generic::BlockData<
	<B as BlockT>::Header,
	<B as BlockT>::Hash,
	<B as BlockT>::Extrinsic,
>;

/// Type alias for using the BlockResponse type using block type parameters.
pub type BlockResponse<B> = generic::BlockResponse<
	<B as BlockT>::Header,
	<B as BlockT>::Hash,
	<B as BlockT>::Extrinsic,
>;

/// Type alias for using the BftMessage type using block type parameters.
pub type BftMessage<B> = generic::BftMessage<
	B,
	<B as BlockT>::Hash,
>;

/// Type alias for using the SignedConsensusProposal type using block type parameters.
pub type SignedConsensusProposal<B> = generic::SignedConsensusProposal<
	B,
	<B as BlockT>::Hash,
>;

/// Type alias for using the SignedConsensusProposal type using block type parameters.
pub type SignedConsensusMessage<B> = generic::SignedConsensusProposal<
	B,
	<B as BlockT>::Hash,
>;

/// A set of transactions.
pub type Transactions<E> = Vec<E>;

/// Bits of block data and associated artefacts to request.
bitflags! {
	/// Node roles bitmask.
	pub struct BlockAttributes: u8 {
		/// Include block header.
		const HEADER = 0b00000001;
		/// Include block body.
		const BODY = 0b00000010;
		/// Include block receipt.
		const RECEIPT = 0b00000100;
		/// Include block message queue.
		const MESSAGE_QUEUE = 0b00001000;
		/// Include a justification for the block.
		const JUSTIFICATION = 0b00010000;
	}
}

impl Encode for BlockAttributes {
	fn encode_to<T: Output>(&self, dest: &mut T) {
		dest.push_byte(self.bits())
	}
}

impl Decode for BlockAttributes {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Self::from_bits(input.read_byte()?)
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode)]
/// Block enumeration direction.
pub enum Direction {
	/// Enumerate in ascending order (from child to parent).
	Ascending = 0,
	/// Enumerate in descendfing order (from parent to canonical child).
	Descending = 1,
}

/// Remote call response.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct RemoteCallResponse {
	/// Id of a request this response was made for.
	pub id: RequestId,
	/// Execution proof.
	pub proof: Vec<Vec<u8>>,
}

/// Generic types.
pub mod generic {
	use primitives::AuthorityId;
	use runtime_primitives::bft::Justification;
	use ed25519;
	use service::Roles;
	use super::{BlockAttributes, RemoteCallResponse, RequestId, Transactions, Direction};


	/// Block data sent in the response.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockData<Header, Hash, Extrinsic> {
		/// Block header hash.
		pub hash: Hash,
		/// Block header if requested.
		pub header: Option<Header>,
		/// Block body if requested.
		pub body: Option<Vec<Extrinsic>>,
		/// Block receipt if requested.
		pub receipt: Option<Vec<u8>>,
		/// Block message queue if requested.
		pub message_queue: Option<Vec<u8>>,
		/// Justification if requested.
		pub justification: Option<Justification<Hash>>,
	}

	/// Identifies starting point of a block sequence.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum FromBlock<Hash, Number> {
		/// Start with given hash.
		Hash(Hash),
		/// Start with given block number.
		Number(Number),
	}

	/// Communication that can occur between participants in consensus.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum BftMessage<Block, Hash> {
		/// A consensus message (proposal or vote)
		Consensus(SignedConsensusMessage<Block, Hash>),
		/// Auxiliary communication (just proof-of-lock for now).
		Auxiliary(Justification<Hash>),
	}

	/// BFT Consensus message with parent header hash attached to it.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct LocalizedBftMessage<Block, Hash> {
		/// Consensus message.
		pub message: BftMessage<Block, Hash>,
		/// Parent header hash.
		pub parent_hash: Hash,
	}

	/// A localized proposal message. Contains two signed pieces of data.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct SignedConsensusProposal<Block, Hash> {
		/// The round number.
		pub round_number: u32,
		/// The proposal sent.
		pub proposal: Block,
		/// The digest of the proposal.
		pub digest: Hash,
		/// The sender of the proposal
		pub sender: AuthorityId,
		/// The signature on the message (propose, round number, digest)
		pub digest_signature: ed25519::Signature,
		/// The signature on the message (propose, round number, proposal)
		pub full_signature: ed25519::Signature,
	}

	/// A localized vote message, including the sender.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct SignedConsensusVote<H> {
		/// The message sent.
		pub vote: ConsensusVote<H>,
		/// The sender of the message
		pub sender: AuthorityId,
		/// The signature of the message.
		pub signature: ed25519::Signature,
	}

	/// Votes during a consensus round.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum ConsensusVote<H> {
		/// Prepare to vote for proposal with digest D.
		Prepare(u32, H),
		/// Commit to proposal with digest D..
		Commit(u32, H),
		/// Propose advancement to a new round.
		AdvanceRound(u32),
	}

	/// A localized message.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum SignedConsensusMessage<Block, Hash> {
		/// A proposal.
		Propose(SignedConsensusProposal<Block, Hash>),
		/// A vote.
		Vote(SignedConsensusVote<Hash>),
	}

	/// A network message.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub enum Message<Block, Header, Hash, Number, Extrinsic> {
		/// Status packet.
		Status(Status<Hash, Number>),
		/// Block request.
		BlockRequest(BlockRequest<Hash, Number>),
		/// Block response.
		BlockResponse(BlockResponse<Header, Hash, Extrinsic>),
		/// Block announce.
		BlockAnnounce(BlockAnnounce<Header>),
		/// Transactions.
		Transactions(Transactions<Extrinsic>),
		/// BFT Consensus statement.
		BftMessage(LocalizedBftMessage<Block, Hash>),
		/// Remote method call request.
		RemoteCallRequest(RemoteCallRequest<Hash>),
		/// Remote method call response.
		RemoteCallResponse(RemoteCallResponse),
		/// Chain-specific message
		#[codec(index = "255")]
		ChainSpecific(Vec<u8>),
	}

	/// Status sent on connection.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct Status<Hash, Number> {
		/// Protocol version.
		pub version: u32,
		/// Supported roles.
		pub roles: Roles,
		/// Best block number.
		pub best_number: Number,
		/// Best block hash.
		pub best_hash: Hash,
		/// Genesis block hash.
		pub genesis_hash: Hash,
		/// Chain-specific status.
		pub chain_status: Vec<u8>,
	}

	/// Request block data from a peer.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockRequest<Hash, Number> {
		/// Unique request id.
		pub id: RequestId,
		/// Bits of block data to request.
		pub fields: BlockAttributes,
		/// Start from this block.
		pub from: FromBlock<Hash, Number>,
		/// End at this block. An implementation defined maximum is used when unspecified.
		pub to: Option<Hash>,
		/// Sequence direction.
		pub direction: Direction,
		/// Maximum number of blocks to return. An implementation defined maximum is used when unspecified.
		pub max: Option<u32>,
	}

	/// Response to `BlockRequest`
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockResponse<Header, Hash, Extrinsic> {
		/// Id of a request this response was made for.
		pub id: RequestId,
		/// Block data for the requested sequence.
		pub blocks: Vec<BlockData<Header, Hash, Extrinsic>>,
	}

	/// Announce a new complete relay chain block on the network.
	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	pub struct BlockAnnounce<H> {
		/// New block header.
		pub header: H,
	}

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
	/// Remote call request.
	pub struct RemoteCallRequest<H> {
		/// Unique request id.
		pub id: RequestId,
		/// Block at which to perform call.
		pub block: H,
		/// Method name.
		pub method: String,
		/// Call data.
		pub data: Vec<u8>,
	}
}
