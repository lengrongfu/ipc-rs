//! Module containing raw definitions mapping to the underlying IPC protocol

use Message;

use libc::{
	size_t,
	IPC_CREAT,
	IPC_EXCL,
	IPC_NOWAIT,
	IPC_RMID,
	IPC_SET,
	IPC_STAT,
	IPC_INFO,
	MSG_INFO,
	MSG_STAT,
	MSG_NOERROR,
	MSG_EXCEPT,
	MSG_COPY,
};

use std::os::raw::{c_int, c_long};

extern "C" {
	/// Raw `msgsnd()` function; we need to use this declaration as opposed to
	/// the one in the `libc` crate due to `c_void` type bullshittery.
	/// Keep in mind that due to the lifetime implications of `*const` pointers
	/// and lifetimes, you should create the raw pointer during the function
	/// call (or as late as possible) so that you avoid dangling pointers
	///
	/// Parameters:
	/// * `msqid` - ID of the message queue in question
	/// * `msgp` - pointer to a user-defined message struct, such struct is
	///    required to have two fields: `mtype` and `mtext`, the first being
	///    a `long`/`i64` which should be positive and the latter an array
	///    or struct (but not a pointer) as chosen by the user
	/// * `msgsz` - maximum size of message (only mtext is counted) that can
	///    be read; depending on configuration, messages larger than `msgsz`
	///    will be either truncated (MSG_NOERROR flag) or the function will
	///    fail and set `errno` to `E2BIG` (default behavior)
	/// * `msgflg` - flags, see [`IpcFlags`] for more info
	pub fn msgsnd(msqid: c_int, msgp: *const Message, msgsz: size_t, msgflg: c_int) -> c_int;

	/// Raw `msgrcv()` function; see [`msgsnd()`] for more details on usage
	///
	/// Parameters:
	/// * `msqid` - ID of the message queue in question
	/// * `msgp` - pointer to a user-defined message struct, such struct is
	///    required to have two fields: `mtype` and `mtext`, the first being
	///    a `long`/`i64` which should be positive and the latter an array
	///    or struct (but not a pointer) as chosen by the user
	/// * `msgsz` - maximum size of message (only mtext is counted) that can
	///    be read; depending on configuration, messages larger than `msgsz`
	///    will be either truncated (MSG_NOERROR flag) or the function will
	///    fail and set `errno` to `E2BIG` (default behavior)
	/// * `msgflg` - flags, see [`IpcFlags`] for more info
	pub fn msgrcv(msqid: c_int, msgp: *mut Message, msgsz: size_t, msgtyp: c_long, msgflg: c_int) -> isize;
}


/// Bit flags for `msgget`
#[repr(i32)]
pub enum IpcFlags {
	/// Create the queue if it doesn't exist
	CreateKey  = IPC_CREAT,
	/// Fail if not creating and queue doesn't
	/// exist and fail if creating and queue
	/// already exists
	Exclusive  = IPC_EXCL,
	/// Make `send()` and `recv()` async
	NoWait     = IPC_NOWAIT,
	/// Allow truncation of message data
	MsgNoError = MSG_NOERROR,
	/// Used with `msgtyp` greater than 0
	/// to read the first message in the
	/// queue with a different type than
	/// `msgtyp`
	MsgExcept  = MSG_EXCEPT,
	/// Copy the message instead of removing
	/// it from the queue. Used for `peek()`
	MsgCopy    = MSG_COPY,
}

/// Commands for use with `msgctl()`
#[repr(i32)]
pub enum ControlCommands {
	/// Instantly deletes queue,
	/// sending a signal to all
	/// waiting processes.
	///
	/// In this case, `msqid_ds` is completely ignored
	/// and you can theoretically even pass a null pointer
	/// without having to worry
	DeleteQueue = IPC_RMID,
	/// Write the values of some
	/// members of `msqid_ds` to
	/// the kernel data structure
	///
	/// Parameters written:
	/// * `msg_qbytes` - the maximum amount of
	///   bytes allowed in the queuue. [`msgsnd()`]
	///   calls over the memory limit will block
	///   unless [`IpcFlags::NoWait`] is also specified
	/// * `msg_perm.uid` - UID of the owner of the queue
	///   (not the same as ID of the creator, which is
	///   saved in `msg_perm.cuid` and can't be changed)
	/// * `msg_perm.gid` - group ID of the owner of
	///   the queue (not the same as GID of the creator,
	///   which is saved in `msg_perm.cgid` and can't
	///   be modified)
	/// * the 9 least significant bits of `msg_perm.mode` -
	///   these are the same mode bits you would encouter
	///   when working with files in Unix. Execute bits
	///   are ignored
	SetOptions  = IPC_SET,
	/// Copies information from the kernel data structure
	/// into the `msqid_ds` struct. The caller mus have read
	/// permission on the message queue in the structure
	GetOptions  = IPC_STAT,
	/// Returns systen-wide information about message queue
	/// limits and parameters. In this case, `msgctl()` expects
	/// a `msginfo` struct as a parameter. Therefore, a cast
	/// is required
	///
	/// This command is Linux specific.
	///
	/// `msginfo` has the following members with the following
	/// meanings:
	/// * `int msgpool` - Size in Kibibites of buffer pool
	///   used to hold message data; unused within kernel
	/// * `int msgmap` - maximum number of entries in message
	///   map, also unused
	/// * `int msgmax` - maximum numberr of bytes that can be
	///   written in a single message`
	/// * `int msgmnb` - maximum number of bytes that can be
	///   written to a queue; used to initialize msg_qbytes
	/// * `int msgmni` - maximum number of message queues
	/// * `int msgssz` - message segment size; unused within
	///   the Linux kernel
	/// * `int msgtql` - maximum number of messages on all
	///   queues in system; unused
	/// * `unsigned short int msgseg` - maximum number of
	///   segments; also unusued
	///
	/// The `msgmni`, `msgmax` and `msgmnb` settings can be changed
	/// through `/proc` files of the same name. These are
	/// usually located in `/proc/sys/kernel`
	///
	/// For bigger payloads it is recommended to raise `msgmax`
	ReturnInfo  = IPC_INFO,
	/// Returns a same `msgid_ds` struct as [`ControlCommands::GetOptions`]
	/// does with the exception that the `msqid` argument is not a queue
	/// identifier but instead an index into the kernel's internal array
	///
	/// This command is Linux specific and rarely used outside of kernel.
	MessageStat = MSG_STAT,
	/// Returns `msginfo` struct just like [`ControlCommands::ReturnInfo`]
	/// would do, but instead reports information about system resources
	/// consumed by message queues.
	///
	/// This command is Linux specific.
	///
	/// The changed struct members are:
	/// * `msgpool` - now returns the number of message queues that currently
	///   exist on the system
	/// * `msgmap` - now returns the total number of messages in all queues
	///   across the system
	/// * `msgtql` - now returns the total number of bytes used by messages
	///   accross the system
	MessageInfo = MSG_INFO,
}
