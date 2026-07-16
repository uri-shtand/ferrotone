
* Go over each class. Understand it, review it and split it if needed.
* Use multiple microphone inputs to get better results
* Recording
* A widget containing buttons to produce specific pitches. Ideally in a simulated singing voice
* Add testing with real voice recordings
* Download voice samples (free)
* Add pitch graph over time ✓
* Understand the pitch detection class ✓
* Pitch detection is noisy/jittery ✓ (stabilizer filters octave errors, spikes, and tremor)
* Each note has a range (lower and upper) and not just a single point. And you have to stay in the range for X ms for it to count. ✓ (NoteSegmenter — note quantization + min_note_duration_ms gate)
* Voice Detect button - instructs the user to be quiet. Then sing low then sing high. And it learns about the user
* Profiles - with toml file (support for multiple profiles)
* Download existing open source projects and use them for comparison
* Use the recording feature to create test recordings
* Test CREPE (Convolutional Representation for Pitch Estimation)
* Automatic Singing Transcription ✓ (NoteSegmenter → IPC → TranscriptionStaff)
* Use Mir1K singing dataset - https://zenodo.org/records/3532216
* Investigate https://github.com/RickyL-2000/ROSVOT 
* Investigate https://www.sonicvisualiser.org/tony/
* Add a tall narrow indicator for pitch with markings that show where the notes are and where the singer is.
* Push the configuration panel to the bottom
* Increase the default screen size
* Change window organization completely
* If the pitch is not on the tone, but close - mark the closest pitch and what direction we need to go to get there (up or down)
