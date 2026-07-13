import { useState } from 'react';
import AudioControls from './components/AudioControls';
import PitchDisplay from './components/PitchDisplay';
import { useAudioSettings } from './hooks/useAudioSettings';
import { usePitchCapture } from './hooks/usePitchCapture';
import './App.css';

function App() {
  const { isCapturing, error, latestFrame, start, stop } = usePitchCapture();
  const {
    settings,
    dirty,
    updateAudio,
    updateNoiseCancellation,
    save,
    devices,
    refreshDevices,
  } = useAudioSettings();

  const [showControls, setShowControls] = useState(true);
  const nc = settings.noise_cancellation;
  const audio = settings.audio;

  return (
    <main className="container">
      <div className="header-row">
        <h1>FerroTone</h1>
        <button
          className={`btn btn-gear ${showControls ? 'active' : ''}`}
          onClick={() => {
            setShowControls((v) => !v);
            if (!showControls) refreshDevices();
          }}
          type="button"
          title="Audio Controls"
        >
          &#9881;
        </button>
      </div>
      {error && <div className="error-message">{error}</div>}
      <PitchDisplay
        isCapturing={isCapturing}
        latestFrame={latestFrame}
        onStart={start}
        onStop={stop}
      />
      {showControls && (
        <AudioControls
          inputGain={nc.input_gain}
          rmsGateEnabled={nc.rms_gate_enabled}
          rmsThreshold={nc.rms_threshold}
          confidenceGateEnabled={nc.confidence_gate_enabled}
          confidenceThreshold={nc.confidence_threshold}
          bandpassEnabled={nc.bandpass_enabled}
          bandpassLow={nc.bandpass_low}
          bandpassHigh={nc.bandpass_high}
          noiseCancellationEnabled={nc.enabled}
          deviceName={audio.device_name}
          devices={devices}
          onInputGainChange={(v) => updateNoiseCancellation({ input_gain: v })}
          onRmsGateEnabledChange={(v) => updateNoiseCancellation({ rms_gate_enabled: v })}
          onRmsThresholdChange={(v) => updateNoiseCancellation({ rms_threshold: v })}
          onConfidenceGateEnabledChange={(v) => updateNoiseCancellation({ confidence_gate_enabled: v })}
          onConfidenceThresholdChange={(v) => updateNoiseCancellation({ confidence_threshold: v })}
          onBandpassEnabledChange={(v) => updateNoiseCancellation({ bandpass_enabled: v })}
          onBandpassLowChange={(v) => updateNoiseCancellation({ bandpass_low: v })}
          onBandpassHighChange={(v) => updateNoiseCancellation({ bandpass_high: v })}
          onNoiseCancellationEnabledChange={(v) => updateNoiseCancellation({ enabled: v })}
          onDeviceNameChange={(v) => updateAudio({ device_name: v })}
          onSave={save}
          dirty={dirty}
        />
      )}
    </main>
  );
}

export default App;
