import { useState } from 'react';
import { Cog } from 'lucide-react';
import AudioControls from './components/AudioControls';
import PitchDisplay from './components/PitchDisplay';
import PitchGraph from './components/PitchGraph';
import TranscriptionStaff from './components/TranscriptionStaff';
import VolumeGraph from './components/VolumeGraph';
import { useAudioSettings } from './hooks/useAudioSettings';
import { useNoteCapture } from './hooks/useNoteCapture';
import { usePitchCapture } from './hooks/usePitchCapture';
import { usePitchGraphCapture } from './hooks/usePitchGraphCapture';
import { useVolumeCapture } from './hooks/useVolumeCapture';

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

  const { bufferRef: volumeBufferRef } = useVolumeCapture(isCapturing);
  const { bufferRef: pitchBufferRef } = usePitchGraphCapture(isCapturing);
  const { segmentsRef } = useNoteCapture(isCapturing);

  const [showControls, setShowControls] = useState(true);
  const nc = settings.noise_cancellation;
  const audio = settings.audio;

  return (
    <main className="flex flex-col items-center min-h-screen">
      <div className="max-w-xl w-full px-4 py-8 text-center">
        <div className="flex items-center justify-center gap-3 mb-8">
          <h1 className="text-xl font-light tracking-[0.1em] uppercase text-muted-foreground">
            FerroTone
          </h1>
          <button
            className={`inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 h-9 w-9 border border-muted-foreground/30 text-muted-foreground hover:bg-accent hover:text-accent-foreground ${showControls ? 'bg-primary text-primary-foreground border-primary hover:bg-primary/90 hover:text-primary-foreground' : 'bg-transparent'}`}
            onClick={() => {
              setShowControls((v) => !v);
              if (!showControls) refreshDevices();
            }}
            type="button"
            title="Audio Controls"
          >
            <Cog className="h-4 w-4" />
          </button>
        </div>
        {error && (
          <div className="bg-destructive/15 border border-destructive text-destructive rounded-lg px-4 py-2 mb-4 text-sm">
            {error}
          </div>
        )}
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
            rnnoiseEnabled={nc.rnnoise_enabled}
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
            onRnnoiseEnabledChange={(v) => updateNoiseCancellation({ rnnoise_enabled: v })}
            onDeviceNameChange={(v) => updateAudio({ device_name: v })}
            onSave={save}
            dirty={dirty}
          />
        )}
      </div>
      <VolumeGraph bufferRef={volumeBufferRef} isCapturing={isCapturing} />
      <PitchGraph bufferRef={pitchBufferRef} isCapturing={isCapturing} />
      <TranscriptionStaff segmentsRef={segmentsRef} isCapturing={isCapturing} />
    </main>
  );
}

export default App;
