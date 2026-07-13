* * Sensitivity
* * [ Raw Mic Input ] ──> [ RMS Volume Gate ] ──> [ RNNoise (nnnoiseless) ] ──> [ Pitch Engine ]
* * Start with RMS Volume Gate and Algorithm Confidence Score (YIN and pYIN)
* * Pre-Filtering with a Bandpass Filter
* * WebRTC Noise Suppression / RNNoise

* Go over each class. Understand it, review it and split it if needed.
* Use multiple microphone inputs to get better results
* Recording
* Only care about voice frequency and not other noises (noise fi*lt*ering)
* Add volume graph that shows how loud the singing is over time.
* Bandpass filter high should be according to human voice and not 1000 hz
* Understand why there is jitter
* A widget containing buttons to produce specific pitches. Ideally in a simulated singing voice
* Command line mode
* Add testing with real voice recordings
* Download voice samples (free)
* Add pitch graph over time
