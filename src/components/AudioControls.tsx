import { useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { Button } from './ui/button';
import { Card, CardContent, CardFooter } from './ui/card';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from './ui/collapsible';
import { Label } from './ui/label';
import { Switch } from './ui/switch';
import { Slider } from './ui/slider';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import type { DeviceInfo } from '../types';

interface AudioControlsProps {
  inputGain: number;
  rmsGateEnabled: boolean;
  rmsThreshold: number;
  confidenceGateEnabled: boolean;
  confidenceThreshold: number;
  bandpassEnabled: boolean;
  bandpassLow: number;
  bandpassHigh: number;
  noiseCancellationEnabled: boolean;
  rnnoiseEnabled: boolean;
  deviceName: string;
  devices: DeviceInfo[];
  onInputGainChange: (v: number) => void;
  onRmsGateEnabledChange: (v: boolean) => void;
  onRmsThresholdChange: (v: number) => void;
  onConfidenceGateEnabledChange: (v: boolean) => void;
  onConfidenceThresholdChange: (v: number) => void;
  onBandpassEnabledChange: (v: boolean) => void;
  onBandpassLowChange: (v: number) => void;
  onBandpassHighChange: (v: number) => void;
  onNoiseCancellationEnabledChange: (v: boolean) => void;
  onRnnoiseEnabledChange: (v: boolean) => void;
  onDeviceNameChange: (v: string) => void;
  onSave: () => void;
  dirty: boolean;
}

function Section({
  label,
  open,
  onToggle,
  children,
}: {
  label: string;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  return (
    <Collapsible open={open} onOpenChange={onToggle}>
      <CollapsibleTrigger className="flex w-full items-center gap-2 px-3 py-2.5 text-sm font-semibold uppercase tracking-wider text-muted-foreground hover:bg-accent rounded-md transition-colors [&[data-state=open]>svg]:rotate-180">
        {open ? <ChevronUp className="h-3 w-3 transition-transform" /> : <ChevronDown className="h-3 w-3 transition-transform" />}
        {label}
      </CollapsibleTrigger>
      <CollapsibleContent className="px-3 pb-3 pt-1 space-y-2">
        {children}
      </CollapsibleContent>
    </Collapsible>
  );
}

export default function AudioControls({
  inputGain,
  rmsGateEnabled,
  rmsThreshold,
  confidenceGateEnabled,
  confidenceThreshold,
  bandpassEnabled,
  bandpassLow,
  bandpassHigh,
  noiseCancellationEnabled,
  rnnoiseEnabled,
  deviceName,
  devices,
  onInputGainChange,
  onRmsGateEnabledChange,
  onRmsThresholdChange,
  onConfidenceGateEnabledChange,
  onConfidenceThresholdChange,
  onBandpassEnabledChange,
  onBandpassLowChange,
  onBandpassHighChange,
  onNoiseCancellationEnabledChange,
  onRnnoiseEnabledChange,
  onDeviceNameChange,
  onSave,
  dirty,
}: AudioControlsProps) {
  const [openSections, setOpenSections] = useState<Record<string, boolean>>({
    input: true,
    nc: false,
    rms: false,
    confidence: false,
    bandpass: false,
  });

  const toggle = (key: string) =>
    setOpenSections((prev) => ({ ...prev, [key]: !prev[key] }));

  return (
    <Card className="mt-6">
      <CardContent className="p-2 space-y-0">
        <Section
          label="Input"
          open={openSections.input}
          onToggle={() => toggle('input')}
        >
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Gain</Label>
              <div className="flex-1 flex items-center gap-3">
                <Slider
                  value={[inputGain]}
                  onValueChange={([v]) => onInputGainChange(v)}
                  min={0}
                  max={2}
                  step={0.05}
                  className="flex-1"
                />
                <span className="min-w-[55px] text-right text-xs text-muted-foreground tabular-nums">
                  {inputGain.toFixed(2)}x
                </span>
              </div>
            </div>
            {devices.length > 0 && (
              <div className="flex items-center gap-3">
                <Label className="min-w-[80px] text-xs text-muted-foreground">Device</Label>
                <Select
                  value={deviceName || 'default'}
                  onValueChange={(v) => onDeviceNameChange(v === 'default' ? '' : v)}
                >
                  <SelectTrigger className="flex-1 h-8 text-xs">
                    <SelectValue placeholder="System Default" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">System Default</SelectItem>
                    {devices.map((d) => (
                      <SelectItem key={d.name} value={d.name}>
                        {d.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
          </div>
        </Section>

        <Section
          label="Volume Gate"
          open={openSections.rms}
          onToggle={() => toggle('rms')}
        >
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Enabled</Label>
              <Switch
                checked={rmsGateEnabled}
                onCheckedChange={onRmsGateEnabledChange}
              />
            </div>
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Threshold</Label>
              <div className="flex-1 flex items-center gap-3">
                <Slider
                  value={[rmsThreshold]}
                  onValueChange={([v]) => onRmsThresholdChange(v)}
                  min={0}
                  max={0.5}
                  step={0.001}
                  className="flex-1"
                />
                <span className="min-w-[55px] text-right text-xs text-muted-foreground tabular-nums">
                  {rmsThreshold.toFixed(3)}
                </span>
              </div>
            </div>
          </div>
        </Section>

        <Section
          label="Confidence Gate"
          open={openSections.confidence}
          onToggle={() => toggle('confidence')}
        >
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Enabled</Label>
              <Switch
                checked={confidenceGateEnabled}
                onCheckedChange={onConfidenceGateEnabledChange}
              />
            </div>
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Threshold</Label>
              <div className="flex-1 flex items-center gap-3">
                <Slider
                  value={[confidenceThreshold]}
                  onValueChange={([v]) => onConfidenceThresholdChange(v)}
                  min={0}
                  max={1}
                  step={0.01}
                  className="flex-1"
                />
                <span className="min-w-[55px] text-right text-xs text-muted-foreground tabular-nums">
                  {confidenceThreshold.toFixed(2)}
                </span>
              </div>
            </div>
          </div>
        </Section>

        <Section
          label="Noise Cancellation"
          open={openSections.nc}
          onToggle={() => toggle('nc')}
        >
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Master</Label>
              <Switch
                checked={noiseCancellationEnabled}
                onCheckedChange={onNoiseCancellationEnabledChange}
              />
            </div>
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">RNNoise</Label>
              <Switch
                checked={rnnoiseEnabled}
                onCheckedChange={onRnnoiseEnabledChange}
              />
            </div>
          </div>
        </Section>

        <Section
          label="Bandpass Filter"
          open={openSections.bandpass}
          onToggle={() => toggle('bandpass')}
        >
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Enabled</Label>
              <Switch
                checked={bandpassEnabled}
                onCheckedChange={onBandpassEnabledChange}
              />
            </div>
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">Low Cut</Label>
              <div className="flex-1 flex items-center gap-3">
                <Slider
                  value={[bandpassLow]}
                  onValueChange={([v]) => onBandpassLowChange(v)}
                  min={20}
                  max={2000}
                  step={1}
                  className="flex-1"
                />
                <span className="min-w-[55px] text-right text-xs text-muted-foreground tabular-nums">
                  {bandpassLow.toFixed(0)} Hz
                </span>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <Label className="min-w-[80px] text-xs text-muted-foreground">High Cut</Label>
              <div className="flex-1 flex items-center gap-3">
                <Slider
                  value={[bandpassHigh]}
                  onValueChange={([v]) => onBandpassHighChange(v)}
                  min={100}
                  max={4000}
                  step={1}
                  className="flex-1"
                />
                <span className="min-w-[55px] text-right text-xs text-muted-foreground tabular-nums">
                  {bandpassHigh.toFixed(0)} Hz
                </span>
              </div>
            </div>
          </div>
        </Section>
      </CardContent>
      <CardFooter className="flex items-center justify-between px-3 py-3">
        <Button
          onClick={onSave}
          disabled={!dirty}
          variant={dirty ? 'default' : 'outline'}
          size="sm"
        >
          Save Settings
        </Button>
        <span className="text-xs text-muted-foreground">
          {dirty ? 'Unsaved changes' : 'All changes saved'}
        </span>
      </CardFooter>
    </Card>
  );
}
