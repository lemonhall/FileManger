import { createApp } from 'vue'
import App from './App.vue'
import './style.css' // Optional: Add a basic CSS file if needed
import { createNotivue } from 'notivue' // Import Notivue

// Import Notivue CSS (Order matters: animations first, then notification styles)
import 'notivue/animations.css' 
import 'notivue/notification.css'

const notivue = createNotivue({
  // Global Notivue options (optional)
  // e.g., position: 'top-right', duration: 5000
})

const app = createApp(App)
app.use(notivue) // Register Notivue plugin
app.mount('#app') 