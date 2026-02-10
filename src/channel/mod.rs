//! Real-time safe channel abstractions.
//!
//! This module provides type-safe wrappers around channels that enforce
//! real-time safety at the type level.

use flume::{Receiver, Sender, TrySendError};
use std::fmt;

use crate::error::{AudioEngineError, Result};
use crate::markers::{NonBlocking, RealtimeSafe};

/// Creates a bounded channel pair for control messages.
///
/// The sender is intended for the control thread (non-RT),
/// and the receiver for the real-time thread.
#[must_use]
pub fn control_channel<T>(capacity: usize) -> (ControlSender<T>, RealtimeReceiver<T>) {
    let (tx, rx) = flume::bounded(capacity);
    (ControlSender { inner: tx }, RealtimeReceiver { inner: rx })
}

/// Creates a bounded channel pair for feedback from RT to control thread.
#[must_use]
pub fn feedback_channel<T>(capacity: usize) -> (RealtimeSender<T>, ControlReceiver<T>) {
    let (tx, rx) = flume::bounded(capacity);
    (RealtimeSender { inner: tx }, ControlReceiver { inner: rx })
}

// ============================================================================
// Control Thread -> Real-Time Thread
// ============================================================================

/// Sender end for control messages (non-RT to RT).
///
/// This sender is held by the control/UI thread and sends messages
/// to the real-time thread. It may block if the channel is full.
pub struct ControlSender<T> {
    inner: Sender<T>,
}

impl<T> ControlSender<T> {
    /// Sends a message, blocking if the channel is full.
    ///
    /// # Errors
    /// Returns an error if the receiver has been dropped.
    pub fn send(&self, msg: T) -> Result<()> {
        self.inner
            .send(msg)
            .map_err(|_| AudioEngineError::ChannelSendFailed)
    }

    /// Tries to send a message without blocking.
    ///
    /// # Errors
    /// Returns an error if the channel is full or disconnected.
    pub fn try_send(&self, msg: T) -> Result<()> {
        self.inner.try_send(msg).map_err(|e| match e {
            TrySendError::Full(_) => AudioEngineError::RingBufferFull { count: 1 },
            TrySendError::Disconnected(_) => AudioEngineError::ChannelSendFailed,
        })
    }

    /// Returns true if the receiver has been dropped.
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        self.inner.is_disconnected()
    }

    /// Returns the number of messages in the channel.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T> Clone for ControlSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> fmt::Debug for ControlSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlSender")
            .field("len", &self.len())
            .field("disconnected", &self.is_disconnected())
            .finish()
    }
}

/// Receiver end for control messages (on RT thread).
///
/// This receiver is held by the real-time thread and receives messages
/// from the control/UI thread. It only provides non-blocking operations.
pub struct RealtimeReceiver<T> {
    inner: Receiver<T>,
}

impl<T> RealtimeReceiver<T> {
    /// Tries to receive a message without blocking.
    ///
    /// Returns `None` if no message is available.
    #[must_use]
    pub fn try_recv(&self) -> Option<T> {
        self.inner.try_recv().ok()
    }

    /// Drains all available messages into a vector.
    ///
    /// **Warning**: This allocates! Only use for bounded message counts.
    #[must_use]
    pub fn drain(&self) -> Vec<T> {
        self.inner.drain().collect()
    }

    /// Processes all available messages with a callback.
    ///
    /// This is the preferred way to handle messages on RT threads.
    pub fn process_all<F>(&self, mut f: F)
    where
        F: FnMut(T),
    {
        while let Some(msg) = self.try_recv() {
            f(msg);
        }
    }

    /// Returns true if the sender has been dropped.
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        self.inner.is_disconnected()
    }

    /// Returns the number of messages in the channel.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T: Send + 'static> RealtimeSafe for RealtimeReceiver<T> {}
impl<T> NonBlocking for RealtimeReceiver<T> {}

impl<T> fmt::Debug for RealtimeReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealtimeReceiver")
            .field("len", &self.len())
            .field("disconnected", &self.is_disconnected())
            .finish()
    }
}

// ============================================================================
// Real-Time Thread -> Control Thread
// ============================================================================

/// Sender end for feedback messages (RT to non-RT).
///
/// This sender is held by the real-time thread and sends feedback
/// to the control/UI thread. It only provides non-blocking operations.
pub struct RealtimeSender<T> {
    inner: Sender<T>,
}

impl<T> RealtimeSender<T> {
    /// Tries to send a message without blocking.
    ///
    /// Returns `true` if the message was sent, `false` if the channel is full.
    #[must_use]
    pub fn try_send(&self, msg: T) -> bool {
        self.inner.try_send(msg).is_ok()
    }

    /// Returns true if the receiver has been dropped.
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        self.inner.is_disconnected()
    }

    /// Returns the number of messages in the channel.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T: Send + 'static> RealtimeSafe for RealtimeSender<T> {}
impl<T> NonBlocking for RealtimeSender<T> {}

impl<T> Clone for RealtimeSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> fmt::Debug for RealtimeSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealtimeSender")
            .field("len", &self.len())
            .field("disconnected", &self.is_disconnected())
            .finish()
    }
}

/// Receiver end for feedback messages (on control thread).
///
/// This receiver is held by the control/UI thread and receives feedback
/// from the real-time thread. It may block if desired.
pub struct ControlReceiver<T> {
    inner: Receiver<T>,
}

impl<T> ControlReceiver<T> {
    /// Tries to receive a message without blocking.
    #[must_use]
    pub fn try_recv(&self) -> Option<T> {
        self.inner.try_recv().ok()
    }

    /// Receives a message, blocking if none is available.
    ///
    /// # Errors
    /// Returns an error if the sender has been dropped.
    pub fn recv(&self) -> Result<T> {
        self.inner
            .recv()
            .map_err(|_| AudioEngineError::ChannelRecvFailed)
    }

    /// Receives a message with a timeout.
    ///
    /// # Errors
    /// Returns an error if the timeout expires or the sender is dropped.
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<T> {
        self.inner
            .recv_timeout(timeout)
            .map_err(|_| AudioEngineError::ChannelRecvFailed)
    }

    /// Drains all available messages.
    #[must_use]
    pub fn drain(&self) -> Vec<T> {
        self.inner.drain().collect()
    }

    /// Returns true if the sender has been dropped.
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        self.inner.is_disconnected()
    }

    /// Returns the number of messages in the channel.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T> fmt::Debug for ControlReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlReceiver")
            .field("len", &self.len())
            .field("disconnected", &self.is_disconnected())
            .finish()
    }
}

// ============================================================================
// Control Message Types
// ============================================================================

/// Common control message type for the audio engine.
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Start audio processing
    Start,
    /// Stop audio processing
    Stop,
    /// Pause audio processing
    Pause,
    /// Resume from pause
    Resume,
    /// Set the master gain
    SetGain(crate::types::Gain),
    /// Set the master pan
    SetPan(crate::types::Pan),
    /// Update effect parameter
    SetEffectParam {
        /// Effect identifier
        effect_id: u32,
        /// Parameter identifier
        param_id: u32,
        /// New parameter value
        value: f32,
    },
    /// Enable or disable an effect
    SetEffectEnabled {
        /// Effect identifier
        effect_id: u32,
        /// Whether the effect is enabled
        enabled: bool,
    },
    /// Shutdown the engine
    Shutdown,
}

impl RealtimeSafe for EngineCommand {}

/// Feedback from the audio engine to the control thread.
#[derive(Debug, Clone)]
pub enum EngineFeedback {
    /// Current audio levels
    Levels {
        /// Input level in dB
        input_db: crate::types::Decibels,
        /// Output level in dB
        output_db: crate::types::Decibels,
    },
    /// Current transport position
    Position(crate::types::TransportPosition),
    /// Engine state changed
    StateChanged(EngineState),
    /// Buffer underrun occurred
    Underrun,
    /// Error occurred
    Error(String),
}

/// State of the audio engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineState {
    /// Engine is stopped
    Stopped,
    /// Engine is running
    Running,
    /// Engine is paused
    Paused,
    /// Engine encountered an error
    Error,
}

impl RealtimeSafe for EngineState {}
