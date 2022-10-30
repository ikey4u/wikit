package com.zhqli.wikit

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Clear
import androidx.compose.material.icons.rounded.Search
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import com.zhqli.wikit.ui.theme.WikitMobileTheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue

class MainActivity: ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            WikitMobileTheme {
                Scaffold(
                    topBar = {
                        TopAppBar() {
                            Text(text = stringResource(R.string.app_name))
                        }
                    }
                ) { contentPadding ->
                    Box(modifier = Modifier.padding(contentPadding)) {
                        SearchAppBar()
                    }
                }
            }
        }
    }
}

// SearchAppBar is referenced from https://github.com/RajashekarRaju/compose-actors
@Composable
private fun SearchAppBar() {
    var query: String by rememberSaveable { mutableStateOf("") }
    var showClearIcon by rememberSaveable { mutableStateOf(false) }

    if (query.isEmpty()) {
        showClearIcon = false
    } else if (query.isNotEmpty()) {
        showClearIcon = true
    }

    TextField(
        value = query,
        onValueChange = { onQueryChanged ->
            query = onQueryChanged
            if (onQueryChanged.isNotEmpty()) {
            }
        },
        leadingIcon = {
            Icon(
                imageVector = Icons.Rounded.Search,
                tint = MaterialTheme.colors.onBackground,
                contentDescription = "Search Icon"
            )
        },
        trailingIcon = {
            if (showClearIcon) {
                IconButton(onClick = { query = "" }) {
                    Icon(
                        imageVector = Icons.Rounded.Clear,
                        tint = MaterialTheme.colors.onBackground,
                        contentDescription = "Clear Icon"
                    )
                }
            }
        },
        maxLines = 1,
        colors = TextFieldDefaults.textFieldColors(backgroundColor = Color.Transparent),
        placeholder = { Text(text = stringResource(R.string.search_hint)) },
        textStyle = MaterialTheme.typography.subtitle1,
        singleLine = true,
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Text),
        modifier = Modifier
            .fillMaxWidth()
            .background(color = MaterialTheme.colors.background, shape = RectangleShape)
    )
}
