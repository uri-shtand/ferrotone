* * Sensitivity
* * [ Raw Mic Input ] ──> [ RMS Volume Gate ] ──> [ RNNoise (nnnoiseless) ] ──> [ Pitch Engine ]
* * Start with RMS Volume Gate and Algorithm Confidence Score (YIN and pYIN)
* * Pre-Filtering with a Bandpass Filter
* * WebRTC Noise Suppression / RNNoise

* Go over each class. Understand it, review it and split it if needed.
* Use multiple microphone inputs to get better results
* Recording
* Only care about voice frequency and not other noises (noise fi*lt*ering)
* Recording gate widget
* Add volume graph that shows how loud the singing is over time.
* Bandpass filter high should be according to human voice and not 1000 hz
* Understand why there is jitter
