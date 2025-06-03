<template>
  <div class="story-builder">
    <div class="builder-header">
      <h3>📖 Interactive Story Builder</h3>
      <p>Build branching narratives with multiple paths and outcomes</p>
    </div>
    
    <div class="builder-content">
      <div class="story-editor">
        <h4>🖋️ Story Editor</h4>
        <p>Coming soon! This will allow you to:</p>
        <ul>
          <li>Create story nodes with rich text</li>
          <li>Design choice trees and branching paths</li>
          <li>Add inventory items and requirements</li>
          <li>Test your stories in real-time</li>
          <li>Export stories for use in applications</li>
        </ul>
      </div>
      
      <div class="story-preview">
        <h4>👁️ Live Preview</h4>
        <div class="preview-placeholder">
          <p>Story preview will appear here as you build...</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'StoryBuilder'
}
</script>

<style scoped>
.story-builder {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.builder-header {
  text-align: center;
  margin-bottom: 30px;
}

.builder-header h3 {
  color: #2c3e50;
  margin-bottom: 10px;
}

.builder-content {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.story-editor, .story-preview {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 4px 15px rgba(0,0,0,0.1);
}

.story-editor h4, .story-preview h4 {
  color: #3498db;
  margin-bottom: 15px;
}

.preview-placeholder {
  background: #f8f9fa;
  border: 2px dashed #dee2e6;
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  color: #6c757d;
  font-style: italic;
}
</style>