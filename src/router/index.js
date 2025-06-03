import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import TicketManagement from '../views/TicketManagement.vue'
import DependencyGraph from '../components/DependencyGraph.vue'
import Interactivity from '../views/Interactivity.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/tickets',
    name: 'TicketManagement',
    component: TicketManagement
  },
  {
    path: '/graph',
    name: 'DependencyGraph',
    component: DependencyGraph
  },
  {
    path: '/interactive',
    name: 'Interactivity',
    component: Interactivity
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router