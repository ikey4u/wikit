<template>
<div id="app">
  <el-container direction="vertical">
    <el-header>
      <div class="header">
        <el-menu :default-active="activeIndex" class="el-menu-demo" mode="horizontal" @select="handleSelect">
          <el-menu-item index="1">Wikit</el-menu-item>
          <el-submenu index="2">
            <template slot="title">Tools (WIP)</template>
            <el-menu-item index="2-1">Create MDX</el-menu-item>
            <el-menu-item index="2-2">Parse MDX</el-menu-item>
            <el-menu-item index="2-3">View MDX</el-menu-item>
            <el-submenu index="2-4">
              <template slot="title">More</template>
              <el-menu-item index="2-4-1">item one</el-menu-item>
              <el-menu-item index="2-4-2">item two</el-menu-item>
              <el-menu-item index="2-4-3">item three</el-menu-item>
            </el-submenu>
          </el-submenu>
          <el-menu-item index="4"><a href="https://github.com/ikey4u/wikit" target="_blank">Source Code</a></el-menu-item>
        </el-menu>
        <el-input placeholder="Please input" v-model="input3" class="input-with-select">
          <el-select v-model="select" slot="prepend" placeholder="Select">
            <el-option label="Restaurant" value="1"></el-option>
            <el-option label="Order No." value="2"></el-option>
            <el-option label="Tel" value="3"></el-option>
          </el-select>
          <el-button slot="append" icon="el-icon-search"></el-button>
        </el-input>
      </div>
    </el-header>
    <el-divider></el-divider>
    <el-main>
      <el-scrollbar height="100%">
        <router-view></router-view>
      </el-scrollbar>
    </el-main>
    <el-footer>
      Footer
    </el-footer>
  </el-container>
</div>
</template>

<script>
import Menu from './components/Menu.vue'

export default {
  name: 'App',
  data() {
    return {
      input3: '',
      select: '',
      value: '',
      activeIndex: '1',
    }
  },
  methods: {
    handleSelect(key, keypath) {
    },
  },
  components: {
    Menu,
  }
}
</script>

<style lang="scss">
html, body, #app, .el-container {
  padding: 0px;
  margin: 0px;
  height: 100%;
}
.el-container {
  width: 100%;
  .el-header {
    background: #aaa;
    padding: 0px;
    margin: 10px 5% 10px 5%;
    min-height: 88px;
    .header {
      display: flex;
      flex-direction: column;
      justify-content: space-between;
    }
  }
  .el-main {
    background: #bbb;
  }
  .el-footer {
    background: #fef;
  }
}
</style>
