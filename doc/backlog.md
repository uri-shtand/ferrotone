* * Sensitivity
* * [ Raw Mic Input ] ──> [ RMS Volume Gate ] ──> [ RNNoise (nnnoiseless) ] ──> [ Pitch Engine ]

* Go over each class. Understand it, review it and split it if needed.
* Use multiple microphone inputs to get better results
* Recording
* Only care about voice frequency and not other noises (noise filtering)
* Bandpass filter high should be according to human voice and not 1000 hz
* Understand why there is jitter
* A widget containing buttons to produce specific pitches. Ideally in a simulated singing voice
* Command line mode
* Add testing with real voice recordings
* Download voice samples (free)
* Add pitch graph over time ✓
* Understand the pitch detection class
* Pitch detection is broken right now
* Each note has a range (lower and upper) and not just a single point. And you have to stay in the range for X ms for it to count.
* Voice Detect button - instructs the user to be quiet. Then sing low then sing high. And it learns about the user
* Profiles - with toml file (support for multiple profiles)
* Download existing open source projects and use them for comparison